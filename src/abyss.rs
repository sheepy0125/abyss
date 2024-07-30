use crate::{components::certificate::hash_certificate, state::ClientState};

use anyhow::{anyhow, Context};
use windmark::context::RouteContext;

#[derive(Default)]
pub struct AbyssState {
    top_level_cartas_loaded: Vec<(String, i32)>,
    currently: AbyssMode,
}
#[derive(Default, Clone)]
pub enum AbyssMode {
    #[default]
    FetchingCartas,
    ViewingCarta(i32),
}

pub fn handle_fetching_cartas(client: &mut ClientState) -> anyhow::Result<String> {
    let abyss_state = &mut client.abyss_state;

    Ok("fetching cartas".into())
}

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

    match client.abyss_state.currently.clone() {
        AbyssMode::FetchingCartas => handle_fetching_cartas(&mut client),
        AbyssMode::ViewingCarta(id) => todo!("viewing carta"),
    }
}
