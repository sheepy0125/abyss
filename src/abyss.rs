use crate::{
    components::{
        certificate::{hash_certificate, CERT_HASH_LEN},
        pages::abyss::{
            fetch_cartas::handle_fetching_cartas,
            submit_carta::{handle_submit_confirmation, handle_submit_new},
            view_carta::handle_viewing_carta,
            view_cartas::handle_viewing_cartas,
            write_carta::handle_writing_carta,
        },
    },
    consts::{DEFAULT_CARTA, MAX_FROM_LEN, MAX_LINE_LEN, MAX_TITLE_LEN},
    database::{Carta, DatabaseCache, DATABASE, DATABASE_CACHE},
    i18n::Lang,
    state::ClientState,
};

use anyhow::{anyhow, Context as _};
use std::{collections::VecDeque, sync::Arc};
use twinstar::Document;
use urlencoding::decode;
use windmark::context::RouteContext;

/// Helper function to validate an input's length
fn validate_len(
    client: &mut ClientState,
    input: &str,
    len: usize,
) -> Option<windmark::response::Response> {
    // xxx: does postgresql's bpchar type's length use characters or graphemes?
    // xxx: for now, assume the latter
    if input.len() > len {
        client.abyss_state.to_flash.push(format!(
            "{write_too_long} ({actual_len}/{len}): {input}",
            actual_len = input.len(),
            write_too_long = &client.lang.write_too_long,
            input = input
        ));
        return client.redirect_to_abyss().ok();
    }
    None
}

/// Carta information to show in the listing
pub struct CartaInformation {
    pub id: i32,
    pub carta: Arc<Carta>,
}

#[derive(Default)]
pub struct AbyssState {
    pub top_level_cartas_loaded: VecDeque<CartaInformation>,
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
    pub reply: Option<String>,
}
impl AbyssState {
    pub fn new(lang: &Lang) -> Self {
        Self {
            languages: vec![lang.code.clone()],
            top_level_cartas_loaded: VecDeque::from_iter([CartaInformation {
                id: 1,
                carta: Arc::clone(&DEFAULT_CARTA),
            }]),
            ..Default::default()
        }
    }
}

#[derive(Default, Clone)]
pub enum AbyssMode {
    #[default]
    FetchingCartas,
    WritingCarta,
    ReplyingCarta(String), // uuid
    ViewingCartas,
    ViewingCarta(String), // uuid
}

/// Fetch a carta's title and ID. An id of None designates a random carta to be fetched.
fn fetch_carta(client: &ClientState, id: Option<i32>) -> anyhow::Result<Option<CartaInformation>> {
    let mut database_guard = DATABASE
        .lock()
        .map_err(|_| anyhow!("failed to lock database mutex"))?;

    let carta = if let Some(id) = id {
        Some(database_guard.fetch_carta(id)?)
    } else {
        database_guard.fetch_random_carta(
            &client.abyss_state.languages,
            client
                .abyss_state
                .top_level_cartas_loaded
                .iter()
                .map(|info| info.carta.id),
        )?
    };

    if let Some(carta) = carta {
        let carta = DatabaseCache::insert_cache(&DATABASE_CACHE.carta, &carta.uuid.clone(), carta)?;
        return Ok(Some(CartaInformation {
            id: carta.id,
            carta,
        }));
    }
    Ok(None)
}
/// Peek into the abyss
fn handle_peek_state_change(client: &mut ClientState) -> anyhow::Result<AbyssMode> {
    match fetch_carta(client, None)? {
        Some(carta_info) => {
            client
                .abyss_state
                .top_level_cartas_loaded
                .push_front(carta_info);
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

        if let Some(res) = validate_len(client, query, MAX_LINE_LEN) {
            return Ok(res);
        };

        // Empty to cancel
        if query.is_empty() {
            return client.redirect_to_abyss();
        }

        // Delete command
        if query == client.lang.write_delete_command {
            if line_number <= client.abyss_state.write_state.lines.len() {
                client.abyss_state.write_state.lines.remove(line_number - 1);
            }
            return client.redirect_to_abyss();
        }

        // New line
        if line_number == client.abyss_state.write_state.lines.len() + 1 {
            client.abyss_state.write_state.lines.push(String::new());
        }

        let line = &mut client.abyss_state.write_state.lines[line_number - 1];
        *line = query.to_string();

        return client.redirect_to_abyss();
    }

    Ok(windmark::response::Response::input(
        &client.lang.write_new_line_message,
    ))
}
// Change the from / title field of a carta
fn handle_change_field(
    // Workaround for requiring a mutable borrow for the field
    client: &mut ClientState,
    field: &mut Option<String>,
    max_len: usize,
    context: &RouteContext,
) -> anyhow::Result<windmark::response::Response> {
    log::trace!("client with id {id} is changing a field", id = client.id());

    // User input
    if let Some(query) = context.url.query() {
        let query = decode(query).context("malformed input")?;
        let query = query.trim();

        if let Some(res) = validate_len(client, query, max_len) {
            return Ok(res);
        };

        // Delete command
        if query == client.lang.write_delete_command {
            *field = None;
            return client.redirect_to_abyss();
        }

        *field = Some(query.to_string());
        return client.redirect_to_abyss();
    }

    Ok(windmark::response::Response::input(
        &client.lang.write_new_field_message,
    ))
}
/// Handle reporting a carta
fn handle_report_carta(client: &mut ClientState, uuid: &str) -> anyhow::Result<()> {
    let mut database_guard = DATABASE
        .lock()
        .map_err(|_| anyhow!("failed to lock database mutex"))?;
    database_guard.report_carta(uuid)?;
    client
        .abyss_state
        .to_flash
        .push(client.lang.report_submitted_flash.clone());
    Ok(())
}

/// `/abyss` endpoint
pub fn handle_client_in_abyss(
    context: RouteContext,
    lang: &'static Lang,
    certificate: bool,
) -> anyhow::Result<windmark::response::Response> {
    let identifier = if certificate {
        let mut buf = [0; CERT_HASH_LEN];
        buf.copy_from_slice(
            &hash_certificate(&context.certificate.clone().context("no certificate")?)?[..],
        );
        buf
    } else {
        let code = context
            .parameters
            .get("code")
            .context("no certless code")?
            .as_bytes();

        if code.len() != CERT_HASH_LEN {
            Err(anyhow!("invalid code"))?;
        }

        let mut buf = [0; CERT_HASH_LEN];
        buf.copy_from_slice(code);
        buf
    };

    // Lookup or create new client
    let (id, client) = ClientState::lookup_from_identifier(&identifier)?
        .map(Ok::<_, anyhow::Error>)
        .unwrap_or_else(|| ClientState::init_state(&identifier, lang, certificate))?;
    let mut client = client
        .lock()
        .map_err(|_| anyhow!("failed to lock client mutex"))?;
    client.poke();

    if client.lang.code != lang.code {
        client.update_lang(lang)?;
    }

    log::debug!("handling client with id {id} in abyss");

    // Handle state changes
    if let Some(state) = context.parameters.get("state").map(String::as_str) {
        match state {
            "fetch" => client.abyss_state.currently = AbyssMode::FetchingCartas,
            "peek" => client.abyss_state.currently = handle_peek_state_change(&mut client)?,
            "view" => client.abyss_state.currently = AbyssMode::ViewingCartas,
            "from" => {
                // "totally safe"
                let field =
                    unsafe { &mut *std::ptr::addr_of_mut!(client.abyss_state.write_state.from) };
                return handle_change_field(&mut client, field, MAX_FROM_LEN, &context);
            }
            "title" => {
                // "totally safe"
                let field =
                    unsafe { &mut *std::ptr::addr_of_mut!(client.abyss_state.write_state.title) };
                return handle_change_field(&mut client, field, MAX_TITLE_LEN, &context);
            }
            "write" => {
                if client.abyss_state.write_state.reply.is_some() {
                    client.abyss_state.write_state = Default::default();
                }
                client.abyss_state.write_state.reply = None;
                client.abyss_state.currently = AbyssMode::WritingCarta;
            }
            write_line if state.starts_with("write-") => {
                let line_number = write_line.trim_start_matches("write-").parse::<usize>()?;
                // Ensure line number is in range
                if !(1..=MAX_LINE_LEN).contains(&line_number) {
                    Err(anyhow!("invalid line number"))?;
                }
                return handle_write_line(&mut client, &context, line_number);
            }
            "help" => {
                let flash = client.lang.write_help_flash.clone();
                client.abyss_state.to_flash.push(flash)
            }
            "toggle-line-numbers" => {
                client.abyss_state.write_state.hide_line_numbers =
                    !client.abyss_state.write_state.hide_line_numbers;
            }
            "submit-confirmation" => return handle_submit_confirmation(&mut client),
            "submit" => {
                let reply_uuid = client.abyss_state.write_state.reply.clone();
                return handle_submit_new(&mut client, &context, reply_uuid);
            }
            read_carta if state.starts_with("read-") => {
                let uuid = read_carta.trim_start_matches("read-");
                client.abyss_state.currently = AbyssMode::ViewingCarta(uuid.to_string());
            }
            reply_carta if state.starts_with("reply-") => {
                let uuid = reply_carta.trim_start_matches("reply-");
                if !client
                    .abyss_state
                    .write_state
                    .reply
                    .as_deref()
                    .is_some_and(|reply_uuid| reply_uuid == uuid)
                {
                    client.abyss_state.write_state = Default::default();
                }
                client.abyss_state.write_state.reply = Some(uuid.to_string());
                client.abyss_state.currently = AbyssMode::ReplyingCarta(uuid.to_string());
            }
            report_carta if state.starts_with("report-") => {
                let uuid = report_carta.trim_start_matches("report-");
                handle_report_carta(&mut client, uuid)?;
            }
            _ => (),
        };
        return client.redirect_to_abyss();
    }

    let mut flash_document = Document::new();
    while let Some(flash) = client.abyss_state.to_flash.pop() {
        flash_document.add_text(flash).add_blank_line();
    }
    let body = match client.abyss_state.currently {
        AbyssMode::FetchingCartas => handle_fetching_cartas(&mut client)?,
        AbyssMode::WritingCarta => handle_writing_carta(&mut client, None)?,
        AbyssMode::ReplyingCarta(ref uuid) => {
            let uuid = uuid.clone();
            handle_writing_carta(&mut client, Some(uuid))?
        }
        AbyssMode::ViewingCarta(ref uuid) => {
            let uuid = uuid.clone();
            handle_viewing_carta(&mut client, uuid)?
        }
        AbyssMode::ViewingCartas => handle_viewing_cartas(&mut client)?,
    };
    Ok(windmark::response::Response::success(format!(
        "{flash_document}{body}"
    )))
}
