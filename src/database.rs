//! ORM types for the database

use crate::consts::DATABASE_URL;
use crate::tree::TreeBranch;

use anyhow::{anyhow, Context, Result};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};
use fix_fn::fix_fn;
use serde::Serialize;
use std::{cell::RefCell, rc::Rc};

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PooledPg = PooledConnection<ConnectionManager<PgConnection>>;

/// Establish a pool and database connection from `DATABASE_URL`
pub fn establish_connection() -> Result<PgPool> {
    let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL.to_string());
    let pool = PgPool::builder()
        .max_size(10)
        .build(manager)
        .context("creating postgresql pool and ocnnection manager")?;
    Ok(pool)
}

#[derive(Queryable, Selectable, Serialize, Clone)]
#[diesel(table_name = crate::schema::cartas)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Carta {
    pub id: i32,
    pub user_id: Option<i32>,
    pub parent: Option<i32>,
    pub content: String,           // max len: 2048
    pub modification_code: String, // 6-digit pin
    pub creation: i32,
    pub modification: Option<i32>,
}

pub struct Database {
    pub connection: PooledPg,
}
impl Database {
    /// Fetch a carta from its ID
    pub fn fetch(&mut self, id: i32) -> Result<Carta> {
        use crate::schema::cartas::dsl;
        dsl::cartas
            .find(id)
            .get_result(&mut self.connection)
            .with_context(|| anyhow!("fetching carta with id {id}"))
    }

    /// Fetch a tree of all cartas from a carta ID
    /// fixme: currently untested. i don't know if this will work.
    pub fn fetch_tree(&mut self, id: i32) -> Result<TreeBranch<Carta>> {
        // fixme: this is quite an inefficient solution. we traverse to the top from
        // the starting id and *then* build the tree, not caching any results. more
        // database calls than necessary occur.

        let mut current_node = self.fetch(id)?;

        // Traverse to top of tree
        while let Some(parent_id) = current_node.parent {
            current_node = self.fetch(parent_id)?;
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
         -> Result<()> {
            for child in self_ref.borrow_mut().fetch_children(branch.node.id)? {
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

        Rc::into_inner(tree_ref).context("tree had more than one ref")
    }

    /// Helper function to find all children of a parent
    fn fetch_children(&mut self, id: i32) -> Result<Vec<Carta>> {
        use crate::schema::cartas::dsl;
        dsl::cartas
            .filter(dsl::parent.eq(id))
            .load(&mut self.connection)
            .with_context(|| anyhow!("finding children of carta with id {id}"))
    }
}
