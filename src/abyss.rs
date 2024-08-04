use crate::{
    components::{
        certificate::hash_certificate,
        pages::abyss::{fetch_cartas::handle_fetching_cartas, write_carta::handle_writing_carta},
    },
    database::DATABASE,
    get_lang,
    i18n::{Lang, ENGLISH},
    state::ClientState,
};

use anyhow::{anyhow, Context as _};
use std::collections::VecDeque;
use twinstar::{document::HeadingLevel, Document};
use urlencoding::decode;
use windmark::context::RouteContext;

pub const MAX_LINE_LEN: usize = 256;
pub const MAX_NUM_LINES: usize = 50;

#[derive(Default)]
pub struct AbyssState {
    pub top_level_cartas_loaded: VecDeque<(String, i32)>,
    pub currently: AbyssMode,
    pub to_flash: Vec<String>,
    pub languages: Vec<String>,
    pub write_state: AbyssWriteState,
}
#[derive(Default)]
pub struct AbyssWriteState {
    pub lines: Vec<String>,
    pub hide_line_numbers: bool,
    pub title: Option<String>,
    pub from: Option<String>,
}
impl AbyssState {
    pub fn new(lang: &Lang) -> Self {
        Self {
            languages: vec![lang.code.clone()],
            // lines: ["a"; 50].iter().map(|s| (*s).to_string()).collect(), // debug
            ..Default::default()
        }
    }
}

#[derive(Default, Clone, Copy)]
pub enum AbyssMode {
    #[default]
    FetchingCartas,
    WritingCarta,
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
/// Peek into the abyss
fn handle_peek_state_change(client: &mut ClientState) -> anyhow::Result<AbyssMode> {
    match fetch_random_carta(client)? {
        Some(carta) => {
            client.abyss_state.top_level_cartas_loaded.push_front(carta);
        }
        None => {
            client
                .abyss_state
                .to_flash
                .push(client.lang.no_new_cartas_status.clone());
        }
    };
    Ok(AbyssMode::FetchingCartas)
}
// Write a line into a carta
fn handle_write_line(
    client: &mut ClientState,
    context: &RouteContext,
    line_number: usize,
) -> anyhow::Result<windmark::response::Response> {
    log::trace!(
        "client with id {id} is writing on line {line_number}",
        id = client.id(),
    );

    // User input
    if let Some(query) = context.url.query() {
        let query = decode(query).context("malformed uri encoding for query, expected utf-8")?;
        let query = query.trim();

        if line_number > client.abyss_state.write_state.lines.len() + 1 {
            Err(anyhow!("invalid line number"))?;
        }

        // Empty to cancel
        if query.is_empty() {
            return client.redirect_to_abyss();
        }

        // Delete command
        if query == client.lang.write_delete_command {
            client.abyss_state.write_state.lines.remove(line_number - 1);
            return client.redirect_to_abyss();
        }

        // New line
        if line_number == client.abyss_state.write_state.lines.len() + 1 {
            client.abyss_state.write_state.lines.push(String::new());
        }

        let line = &mut client.abyss_state.write_state.lines[line_number - 1];
        // fixme: validate
        *line = query.to_string();

        return client.redirect_to_abyss();
    }

    Ok(windmark::response::Response::input(
        &client.lang.write_new_line_input,
    ))
}
// Change the from / title field of a carta
fn handle_change_field(
    // Workaround for requiring a mutable borrow for the field
    client: &mut ClientState,
    field: &mut Option<String>,
    context: &RouteContext,
) -> anyhow::Result<windmark::response::Response> {
    log::trace!("client with id {id} is changing a field", id = client.id());

    // User input
    if let Some(query) = context.url.query() {
        // Delete command
        if query == client.lang.write_delete_command {
            *field = None;
            return client.redirect_to_abyss();
        }

        // fixme: validate
        *field = Some(decode(query).context("malformed input")?.to_string());
        return client.redirect_to_abyss();
    }

    Ok(windmark::response::Response::input(
        &client.lang.write_new_field,
    ))
}

// `/abyss` endpoint
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
    client.poke();
    if lang.is_none() {
        return client.redirect_to_abyss();
    }
    client.lang = lang.unwrap_or_else(|| &ENGLISH);

    log::debug!("handling client with id {id} in abyss");

    // Handle state changes
    if let Some(state) = context
        .parameters
        .get("state")
        .map(|str_ref| str_ref.as_str())
    {
        match state {
            "fetch" => client.abyss_state.currently = AbyssMode::FetchingCartas,
            "peek" => client.abyss_state.currently = handle_peek_state_change(&mut client)?,
            "from" => {
                // "totally safe"
                let field =
                    unsafe { &mut *std::ptr::addr_of_mut!(client.abyss_state.write_state.from) };
                return handle_change_field(&mut client, field, &context);
            }
            "title" => {
                // "totally safe"
                let field =
                    unsafe { &mut *std::ptr::addr_of_mut!(client.abyss_state.write_state.title) };
                return handle_change_field(&mut client, field, &context);
            }
            "write" => client.abyss_state.currently = AbyssMode::WritingCarta,
            write_line if state.starts_with("write") => {
                let line_number = write_line.trim_start_matches("write-").parse::<usize>()?;
                let line_number = line_number
                    .checked_sub(1)
                    .context("line number underflow")?
                    + 1;
                if line_number > MAX_NUM_LINES {
                    Err(anyhow!("invalid line number"))?;
                }
                return handle_write_line(&mut client, &context, line_number);
            }
            "help" => {
                let flash = client.lang.write_help_status.clone();
                client.abyss_state.to_flash.push(flash)
            }
            "toggle-line-numbers" => {
                client.abyss_state.write_state.hide_line_numbers =
                    !client.abyss_state.write_state.hide_line_numbers;
            }
            _ => (),
        };
        return client.redirect_to_abyss();
    }

    let mut flash_document = Document::new();
    while let Some(flash) = client.abyss_state.to_flash.pop() {
        flash_document
            .add_heading(HeadingLevel::H3, flash)
            .add_blank_line();
    }
    let body = match client.abyss_state.currently {
        AbyssMode::FetchingCartas => handle_fetching_cartas(&mut client)?,
        AbyssMode::WritingCarta => handle_writing_carta(&mut client)?,
        AbyssMode::ViewingCarta(id) => todo!("viewing carta"),
    };
    Ok(windmark::response::Response::success(format!(
        "{flash_document}{body}"
    )))
}
