use crate::components::certificate::CertHash;

use anyhow::{anyhow, Context};
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub type ClientLookup = HashMap<CertHash, Arc<Mutex<ClientState>>>;
pub type Clients = Arc<Mutex<ClientLookup>>;

lazy_static! {
    pub static ref CLIENTS: Clients = Default::default();
}

pub struct ClientState {}

impl ClientState {
    pub fn lookup_from_certificate(
        cert_hash: &CertHash,
    ) -> anyhow::Result<Option<Arc<Mutex<Self>>>> {
        let guard = CLIENTS
            .lock()
            .map_err(|_| anyhow!("failed locking clients mutex"))?;
        let state = Arc::clone(
            guard
                .get(cert_hash)
                .context("client hasn't initialized state")?,
        );
        Ok(Some(state))
    }
}
