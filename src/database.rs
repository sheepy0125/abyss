//! ORM types for the database

use crate::components::certificate::CERT_HASH_LEN;
use crate::tree::TreeBranch;
use crate::{consts::DATABASE_URL, i18n::Lang};

use anyhow::{anyhow, Context as _};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};
use fix_fn::fix_fn;
use lazy_static::lazy_static;
use rand::distributions::Uniform;
use rand::prelude::Distribution as _;
use rand::thread_rng;
use serde::Serialize;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::MutexGuard;
use std::time::{Duration, Instant};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PooledPg = PooledConnection<ConnectionManager<PgConnection>>;

pub const CACHE_INVALIDATION_SECS: u64 = 3600; // 1 hour

lazy_static! {
    pub static ref DATABASE: Arc<Mutex<Database>> = Arc::new(Mutex::new(Database::new(
        establish_connection()
            .and_then(|pool| pool.get().context("getting pool connection"))
            .expect("establish database connection")
    )));
    pub static ref DATABASE_CACHE: DatabaseCache = Default::default();
}

/// A database cache to avoid storing heap-allocated objects for every user
#[derive(Default)]
pub struct DatabaseCache {
    pub user: Arc<Mutex<HashMap<[u8; CERT_HASH_LEN], Cache<User>>>>,
    pub carta: Arc<Mutex<HashMap<String, Cache<Carta>>>>,
}
pub struct Cache<T> {
    pub creation: Instant,
    pub store: Arc<T>,
}
pub trait CacheKey: Hash + Eq + Clone {}
impl<K> CacheKey for K where K: Hash + Eq + Clone {}
impl DatabaseCache {
    pub type TCache<K: Hash + Eq, T> = Arc<Mutex<HashMap<K, Cache<T>>>>;

    pub fn lookup_cache<K: CacheKey, T>(
        cache: &Self::TCache<K, T>,
        key: &K,
    ) -> anyhow::Result<Option<Arc<T>>> {
        let mut guard: MutexGuard<HashMap<_, Cache<T>>> = cache
            .lock()
            .map_err(|_| anyhow!("failed to lock db cache mutex"))?;

        let mut remove = false;
        let mut store = None;
        if let Some(cache) = guard.get(key) {
            if Instant::now().duration_since(cache.creation)
                > Duration::from_secs(CACHE_INVALIDATION_SECS)
            {
                remove = true;
            } else {
                store = Some(Arc::clone(&cache.store));
            }
        }
        if remove {
            guard.remove(key);
            log::trace!("not found");
            return Ok(None);
        }

        Ok(store)
    }

    pub fn insert_cache<K: CacheKey, T>(
        cache: &Self::TCache<K, T>,
        key: &K,
        store: T,
    ) -> anyhow::Result<Arc<T>> {
        let mut guard: MutexGuard<HashMap<_, Cache<T>>> = cache
            .lock()
            .map_err(|_| anyhow!("failed to lock db cache mutex"))?;

        let cache = Cache {
            creation: Instant::now(),
            store: Arc::new(store),
        };

        let store = Arc::clone(&cache.store);
        guard.insert(key.clone(), cache);

        Ok(store)
    }

    pub fn get_or_else<K: CacheKey, T>(
        cache: &Self::TCache<K, T>,
        key: &K,
        otherwise: &dyn Fn() -> anyhow::Result<T>,
    ) -> anyhow::Result<Arc<T>> {
        if let Some(t) = Self::lookup_cache(cache, key)? {
            return Ok(t);
        }
        Self::insert_cache(cache, key, otherwise()?)
    }
}

/// Establish a pool and database connection from `DATABASE_URL`
pub fn establish_connection() -> anyhow::Result<PgPool> {
    log::trace!("initializing database connection");

    let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL.to_string());

    let pool = PgPool::builder()
        .max_size(10)
        .build(manager)
        .context("creating postgresql pool and ocnnection manager")?;

    Ok(pool)
}

#[derive(Queryable, Selectable, Serialize, Clone, Debug)]
#[diesel(table_name = crate::schema::cartas)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Carta {
    pub id: i32,
    pub uuid: String, // 32-len + 4 hyphens
    pub user_id: Option<i32>,
    pub parent: Option<i32>,
    pub title: Option<String>,     // max len: 24
    pub sender: Option<String>,    // max len: 12
    pub content: String,           // max len: 2048
    pub modification_code: String, // 6-digit pin
    pub creation: i32,             // unix timestamp
    pub modification: Option<i32>, // unix timestamp
    pub lang: String,              // 2-digit code, e.g. `en`
    pub random_accessible: bool,
}
#[derive(Insertable, Serialize, Clone, Debug)]
#[diesel(table_name = crate::schema::cartas)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CartaUpdate {
    pub user_id: Option<i32>,
    pub parent: Option<i32>,
    pub uuid: String,              // 32-len + 4 hyphens
    pub title: Option<String>,     // max len: 24
    pub sender: Option<String>,    // max len: 12
    pub content: String,           // max len: 2048
    pub modification_code: String, // 6-digit pin
    pub creation: i32,             // unix timestamp
    pub modification: Option<i32>, // unix timestamp
    pub lang: String,              // 2 digit code, e.g. `en`
    pub random_accessible: bool,
}
pub const MAX_TITLE_LEN: usize = 24;
pub const MAX_FROM_LEN: usize = 12;

#[derive(Queryable, Selectable, Serialize, Clone, Debug)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub certificate_hash: Vec<u8>, // max len: [`crate::certificate::CERT_HASH_LEN`]
    pub lang: String,
    pub creation: i32, // unix timestamp
}
#[derive(Insertable, Serialize, Clone, Debug)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserUpdate {
    pub certificate_hash: Vec<u8>,
    pub lang: String,
    pub creation: i32,
}

pub struct Database {
    pub connection: PooledPg,
}
impl Database {
    pub fn new(connection: PooledPg) -> Self {
        Self { connection }
    }

    /// Fetch a random "random accessible" carta
    pub fn fetch_random_carta<I>(
        &mut self,
        languages: &[String],
        ignore_ids: I,
    ) -> anyhow::Result<Option<Carta>>
    where
        I: Iterator<Item = i32>,
    {
        log::trace!("fetching a random carta from languages {languages:?}");

        diesel::define_sql_function!(fn random() -> Text);

        use crate::schema::cartas::dsl;
        let random_carta = dsl::cartas
            .filter(dsl::random_accessible.eq(true))
            .filter(dsl::id.ne_all(ignore_ids))
            .filter(dsl::lang.eq_any(languages))
            .select(Carta::as_select())
            .order(random())
            .first(&mut self.connection)
            .optional()?;

        Ok(random_carta)
    }

    /// Fetch a user from their certificate hash
    pub fn fetch_user(&mut self, cert_hash: &[u8]) -> anyhow::Result<Option<User>> {
        log::trace!("looking up user from cert hash");

        use crate::schema::users::dsl;
        let user = dsl::users
            .filter(dsl::certificate_hash.eq(cert_hash))
            .select(User::as_select())
            .first(&mut self.connection)
            .optional()
            .with_context(|| anyhow!("fetching user by cert hash"))?;

        if let Some(user) = user {
            log::trace!(
                "user created on {creation} has id {id}",
                id = user.id,
                creation = user.creation
            );
            Ok(Some(user))
        } else {
            log::trace!("certificate not found");
            Ok(None)
        }
    }

    /// Insert a new user
    pub fn insert_user(&mut self, lang: String, cert_hash: &[u8]) -> anyhow::Result<User> {
        log::trace!("inserting a new user");

        let update = UserUpdate {
            certificate_hash: cert_hash.to_vec(),
            creation: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as _,
            lang,
        };

        use crate::schema::users::dsl;
        let user = update
            .insert_into(dsl::users)
            .returning(User::as_returning())
            .get_result(&mut self.connection)?;

        log::trace!("inserted user {id}", id = user.id);

        Ok(user)
    }

    /// Insert a new carta
    pub fn insert_carta(
        &mut self,
        user_id: Option<i32>,
        parent: Option<i32>,
        content: String,
        title: Option<String>,
        from: Option<String>,
        lang: &Lang,
    ) -> anyhow::Result<Carta> {
        log::trace!("inserting a new carta");

        // Generate 6-digit modification PIN
        let uniform = Uniform::new('0', '9');
        let mut rng = thread_rng();
        let modification_code = (0..6).map(|_| uniform.sample(&mut rng)).collect();

        let update = CartaUpdate {
            uuid: uuid::Uuid::new_v4().to_string(),
            user_id,
            parent,
            title,
            sender: from,
            content,
            lang: lang.code.clone(),
            random_accessible: parent.is_none(),
            creation: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as _,
            modification: None,
            modification_code,
        };

        use crate::schema::cartas::dsl;
        let carta = update
            .insert_into(dsl::cartas)
            .returning(Carta::as_returning())
            .get_result(&mut self.connection)?;

        log::trace!("inserted carta {id}", id = carta.id);

        Ok(carta)
    }

    /// Fetch a carta from its ID
    pub fn fetch_carta(&mut self, id: i32) -> anyhow::Result<Carta> {
        log::trace!("fetching carta with id {id}");

        use crate::schema::cartas::dsl;
        let carta = dsl::cartas
            .find(id)
            .get_result(&mut self.connection)
            .with_context(|| anyhow!("fetching carta with id {id}"))?;

        log::trace!("fetched carta with id {id}: {carta:?}");

        Ok(carta)
    }

    /// Fetch a carta from its UUID
    pub fn fetch_carta_uuid(&mut self, uuid: &str) -> anyhow::Result<Carta> {
        log::trace!("fetching carta with uuid {uuid}");

        use crate::schema::cartas::dsl;
        let carta = dsl::cartas
            .filter(dsl::uuid.eq(uuid))
            .select(Carta::as_select())
            .first(&mut self.connection)
            .with_context(|| anyhow!("fetching carta with uuid {uuid}"))?;

        log::trace!("fetched carta with id {id}: {carta:?}", id = carta.id);

        Ok(carta)
    }

    /// Fetch a tree of all cartas from a carta ID
    /// fixme: currently untested. i don't know if this will work.
    pub fn fetch_carta_tree(&mut self, id: i32) -> anyhow::Result<TreeBranch<Carta>> {
        // fixme: this is quite an inefficient solution. we traverse to the top from
        // the starting id and *then* build the tree, not caching any results. more
        // database calls than necessary occur.

        log::trace!("fetching tree off cartas from carta with id {id}");

        let mut current_node = self.fetch_carta(id)?;

        // Traverse to top of tree
        while let Some(parent_id) = current_node.parent {
            current_node = self.fetch_carta(parent_id)?;
        }
        let tree = TreeBranch::<Carta> {
            node: current_node,
            parent: None,
            children: vec![].into(),
        };

        // DFS to build tree
        let self_ref = RefCell::new(self);
        let traverse_downward = fix_fn!(|traverse_downward,
                                         branch: Rc<TreeBranch<Carta>>|
         -> anyhow::Result<()> {
            log::trace!("traversing downwawrd from branch {branch:?}");

            for child in self_ref.borrow_mut().fetch_carta_children(branch.node.id)? {
                let child_branch = TreeBranch {
                    node: child,
                    parent: Some(Rc::downgrade(&branch)),
                    children: vec![].into(),
                };
                let child_branch_ref = Rc::new(child_branch);
                branch
                    .children
                    .borrow_mut()
                    .push(Rc::downgrade(&child_branch_ref));
                traverse_downward(child_branch_ref)?;
            }

            Ok(())
        });

        let tree_ref = Rc::new(tree);
        traverse_downward(Rc::clone(&tree_ref))?;
        log::trace!("tree: {tree_ref:?}");

        Rc::into_inner(tree_ref).context("tree had more than one ref")
    }

    /// Helper function to find all children of a parent
    fn fetch_carta_children(&mut self, id: i32) -> anyhow::Result<Vec<Carta>> {
        use crate::schema::cartas::dsl;
        dsl::cartas
            .filter(dsl::parent.eq(id))
            .load(&mut self.connection)
            .with_context(|| anyhow!("finding children of carta with id {id}"))
    }
}
