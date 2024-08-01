use crate::abyss::AbyssState;
use crate::components::certificate::{CertHash, CERT_HASH_LEN};
use crate::database::DATABASE;
use crate::i18n::Lang;

use anyhow::{anyhow, Context};
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant},
};

pub type ClientLookup = HashMap<[u8; CERT_HASH_LEN], (usize, Arc<Mutex<ClientState>>)>;
pub type Clients = Arc<RwLock<ClientLookup>>;

pub const PRUNE_TIME: Duration = Duration::from_secs(60 * 60); // 1 hour

lazy_static! {
    pub static ref CLIENTS: Clients = Default::default();
}

pub struct ClientState {
    keepalive: Instant,
    id: usize,
    pub abyss_state: AbyssState,
    pub lang: &'static Lang,
}
impl ClientState {
    fn new(cert_hash: &[u8], lang: &'static Lang) -> anyhow::Result<Self> {
        let mut database_guard = DATABASE
            .lock()
            .map_err(|_| anyhow!("failed to lock database mutex"))?;

        let user = database_guard.fetch_user(cert_hash)?.map_or_else(
            || database_guard.insert_user(lang.code.clone(), cert_hash),
            Ok,
        )?;

        Ok(Self {
            keepalive: Instant::now(),
            id: user.id as _,
            abyss_state: AbyssState::new(lang),
            lang,
        })
    }
}
impl ClientState {
    pub fn poke(&mut self) {
        self.keepalive = Instant::now();
    }
    pub fn keeaplive(&self) -> &Instant {
        &(self.keepalive)
    }
    pub fn id(&self) -> usize {
        self.id
    }
}

impl ClientState {
    /// Create a new client
    pub fn init_state(
        cert_hash: &CertHash,
        lang: &'static Lang,
    ) -> anyhow::Result<(usize, Arc<Mutex<Self>>)> {
        log::trace!("creating a new client");

        let mut guard = CLIENTS
            .write()
            .map_err(|_| anyhow!("failed locking clients rwlock"))?;

        let hash = {
            let mut heap_clone = [0u8; CERT_HASH_LEN];
            heap_clone.copy_from_slice(cert_hash);
            heap_clone
        };
        let state = ClientState::new(&hash, lang)?;
        let id = state.id();
        let wrapped_state = (id, Arc::new(Mutex::new(state)));
        guard.insert(hash, wrapped_state.clone());

        log::trace!("created a new client with id {id}");

        Ok(wrapped_state)
    }

    /// Look up a client from the global map
    pub fn lookup_from_certificate(
        cert_hash: &CertHash,
    ) -> anyhow::Result<Option<(usize, Arc<Mutex<Self>>)>> {
        log::trace!("looking up a client from their certificate");

        let guard = CLIENTS
            .write()
            .map_err(|_| anyhow!("failed locking clients rwlock"))?;

        if let Some((id, state)) = guard.get(&cert_hash[..]).cloned() {
            log::trace!("found client with id {id}");
            return Ok(Some((id, state)));
        };
        log::trace!("client not found");
        Ok(None)
    }

    /// Prune clients
    pub fn prune_clients() -> anyhow::Result<()> {
        let guard = CLIENTS
            .read()
            .map_err(|_| anyhow!("failed locking clients rwlock"))?;

        // Find clients to prune first. We only read from the RwLock in this stage.
        let to_prune = guard
            .iter()
            .map(|(cert_ref, (id, client))| {
                let guard = client
                    .lock()
                    .map_err(|_| anyhow!("failed to lock client mutex"))?;
                let lifetime = guard.keeaplive().elapsed();
                Ok(if lifetime > PRUNE_TIME {
                    log::trace!(
                        "client with id {id} has a lifetime of {lifetime}s. pruning",
                        lifetime = lifetime.as_secs()
                    );
                    Some(Box::new(*cert_ref))
                } else {
                    None
                })
            })
            .collect::<anyhow::Result<Vec<Option<_>>>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        if to_prune.is_empty() {
            log::trace!("nothing to prune");
            return Ok(());
        }

        drop(guard);

        // Write to the RwLock to remove the pruned keys
        let mut guard = CLIENTS
            .write()
            .map_err(|_| anyhow!("failed to lock clients rwlock"))?;
        for cert_ref in to_prune {
            let (id, _state) = guard.remove(&cert_ref[..]).context("pruning client")?;
            log::trace!("pruning client with id {id}");
        }
        Ok(())
    }
}
