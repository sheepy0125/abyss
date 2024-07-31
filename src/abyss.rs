use std::collections::VecDeque;

use crate::{
    components::certificate::hash_certificate,
    database::DATABASE,
    get_lang,
    i18n::{Lang, ENGLISH},
    lookup_lang_from_code,
    state::ClientState,
};

use anyhow::{anyhow, Context};
use twinstar::{document::HeadingLevel, Document};
use windmark::context::RouteContext;

pub struct AbyssState {
    top_level_cartas_loaded: VecDeque<(String, i32)>,
    currently: AbyssMode,
    to_flash: Option<String>,
    languages: Vec<String>,
}
impl AbyssState {
    pub fn new(lang: &Lang) -> Self {
        Self {
            top_level_cartas_loaded: VecDeque::new(),
            currently: AbyssMode::default(),
            to_flash: None,
            languages: vec![lang.code.clone()],
        }
    }
}

#[derive(Default, Clone)]
pub enum AbyssMode {
    #[default]
    FetchingCartas,
    ViewingCarta(i32),
}

/// Fetch a random carta's title and ID
fn fetch_random_carta(client: &ClientState) -> anyhow::Result<Option<(String, i32)>> {
    let mut guard = DATABASE
        .lock()
        .map_err(|_| anyhow!("failed to lock database mutex"))?;

    let carta = guard.fetch_random_carta(
        &client.abyss_state.languages,
        client
            .abyss_state
            .top_level_cartas_loaded
            .iter()
            .map(|(_title, id)| *id),
    )?;

    if let Some(carta) = carta {
        return Ok(Some((
            carta.title.unwrap_or_else(|| "untitled".to_string()),
            carta.id,
        )));
    }
    Ok(None)
}

fn handle_fetching_cartas(client: &mut ClientState) -> anyhow::Result<String> {
    let abyss_state = &mut client.abyss_state;

    // Fetch UI
    let fetch_ui = Document::new()
        .add_heading(HeadingLevel::H2, &client.lang.fetch_header)
        .add_link("?peek", &client.lang.fetch)
        .to_string();

    #[allow(clippy::useless_format)]
    Ok(format!("{fetch_ui}"))
}

pub fn handle_client_in_abyss(
    context: RouteContext,
) -> anyhow::Result<windmark::response::Response> {
    let cert_hash = hash_certificate(&context.certificate.clone().context("no certificate")?)?;

    // I18N
    let lang = get_lang(&context);

    // Lookup or create new client
    let (id, client) = ClientState::lookup_from_certificate(&cert_hash)?
        .map(Ok::<_, anyhow::Error>)
        .unwrap_or_else(|| ClientState::init_state(&cert_hash, lang.unwrap_or_else(|| &ENGLISH)))?;
    let mut client = client
        .lock()
        .map_err(|_| anyhow!("failed to lock client mutex"))?;
    if lang.is_none() {
        return Ok(windmark::response::Response::permanent_redirect(format!(
            "{lang}/abyss",
            lang = client.lang.code
        )));
    }
    client.lang = lang.unwrap_or_else(|| &ENGLISH);

    log::debug!("handling client with id {id} in abyss");

    // Handle state changes
    if match context
        .parameters
        .get("state")
        .map(|str_ref| str_ref.as_str())
    {
        Some("peek") => {
            client.abyss_state.currently = AbyssMode::FetchingCartas;
            match fetch_random_carta(&client)? {
                Some(carta) => {
                    client.abyss_state.top_level_cartas_loaded.push_front(carta);
                }
                None => {
                    client.abyss_state.to_flash = Some(client.lang.no_new_cartas.clone());
                }
            };
            true
        }
        _unknown_or_none => false,
    } {
        return Ok(windmark::response::Response::temporary_redirect("/abyss"));
    }

    let flash = match client.abyss_state.to_flash {
        Some(ref to_flash) => Document::new()
            .add_heading(HeadingLevel::H3, to_flash)
            .add_blank_line()
            .to_string(),
        None => "".to_string(),
    };
    client.abyss_state.to_flash = None;

    let body = match client.abyss_state.currently.clone() {
        AbyssMode::FetchingCartas => handle_fetching_cartas(&mut client)?,
        AbyssMode::ViewingCarta(id) => todo!("viewing carta"),
    };
    Ok(windmark::response::Response::success(format!(
        "{flash}{body}"
    )))
}
