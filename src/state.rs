use crate::components::certificate::{CertHash, CERT_HASH_LEN};

use anyhow::{anyhow, Context};
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{atomic::AtomicUsize, Arc, Mutex},
    time::Instant,
};

pub type ClientLookup = HashMap<[u8; CERT_HASH_LEN], (usize, Arc<Mutex<ClientState>>)>;
pub type Clients = Arc<Mutex<ClientLookup>>;

lazy_static! {
    pub static ref CLIENTS: Clients = Default::default();
}

pub static NEXT_CLIENT_ID: AtomicUsize = AtomicUsize::new(0);

pub struct ClientState {
    creation: Instant,
    id: usize,
}
impl Default for ClientState {
    fn default() -> Self {
        Self {
            creation: Instant::now(),
            id: NEXT_CLIENT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        }
    }
}
impl ClientState {
    pub fn creation(&self) -> &Instant {
        &(self.creation)
    }
    pub fn id(&self) -> usize {
        self.id
    }
}

impl ClientState {
    /// Create a new client
    pub fn init_state(cert_hash: &CertHash) -> anyhow::Result<(usize, Arc<Mutex<Self>>)> {
        log::trace!("creating a new client");

        let mut guard = CLIENTS
            .lock()
            .map_err(|_| anyhow!("failed locking clients mutex"))?;

        let state = ClientState::default();
        let id = state.id();
        let wrapped_state = (id, Arc::new(Mutex::new(state)));

        let hash = {
            let mut heap_clone = [0u8; CERT_HASH_LEN];
            heap_clone.copy_from_slice(cert_hash);
            heap_clone
        };
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
            .lock()
            .map_err(|_| anyhow!("failed locking clients mutex"))?;

        if let Some((id, state)) = guard.get(&cert_hash[..]).cloned() {
            log::trace!("found client with id {id}");
            return Ok(Some((id, state)));
        };
        log::trace!("client not found");
        Ok(None)
    }

    /// Prune clients
    pub fn prune_clients() -> anyhow::Result<()> {
        todo!("change mutex to rwlock")
    }
}
