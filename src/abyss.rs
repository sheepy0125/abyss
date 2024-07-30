use anyhow::{anyhow, Context};
use windmark::context::RouteContext;

use crate::{components::certificate::hash_certificate, state::ClientState};

pub fn handle_client_in_abyss(context: RouteContext) -> anyhow::Result<String> {
    let cert_hash = hash_certificate(&context.certificate.context("no certificate")?)?;

    // Lookup or create new client
    let (id, client) = ClientState::lookup_from_certificate(&cert_hash)?
        .map(Ok::<_, anyhow::Error>)
        .unwrap_or_else(|| ClientState::init_state(&cert_hash))?;
    let mut client = client
        .lock()
        .map_err(|_| anyhow!("failed to lock client mutex"))?;

    log::debug!("handling client with id {id} in abyss");

    todo!()
}
