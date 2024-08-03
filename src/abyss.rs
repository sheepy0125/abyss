use crate::{
    components::certificate::hash_certificate,
    database::DATABASE,
    get_lang,
    i18n::{Lang, ENGLISH},
    state::ClientState,
};

use anyhow::{anyhow, Context};
use lazy_static::lazy_static;
use std::collections::VecDeque;
use twinstar::{document::HeadingLevel, Document};
use urlencoding::decode;
use windmark::context::RouteContext;

pub const MAX_LINE_LEN: usize = 256;
pub const MAX_NUM_LINES: usize = 50;

lazy_static! {
    // Twinstar's [`twinstar::Document`] type only allows URIs added through
    // [`Document::add_link`] to have a 'static lifetime.
    pub static ref WRITE_CHANGE_LINKS_LOOKUP_FROM_LINE_NUMBER: [&'static str; MAX_NUM_LINES + 1] = {
        std::array::from_fn(|n| format!("write-{n}").leak() as &'static str)
    };
}

#[derive(Default)]
pub struct AbyssState {
    pub top_level_cartas_loaded: VecDeque<(String, i32)>,
    pub currently: AbyssMode,
    pub to_flash: Vec<String>,
    pub languages: Vec<String>,
    pub lines: Vec<String>,
    pub show_writing_help: bool,
}
impl AbyssState {
    pub fn new(lang: &Lang) -> Self {
        Self {
            languages: vec![lang.code.clone()],
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

fn handle_fetching_cartas(client: &mut ClientState) -> anyhow::Result<String> {
    let abyss_state = &mut client.abyss_state;

    // Fetch UI
    let fetch_ui = Document::new()
        .add_heading(HeadingLevel::H2, &client.lang.fetch_header)
        .add_link("peek", &client.lang.fetch_link)
        .add_link("write", &client.lang.write_link)
        .to_string();

    #[allow(clippy::useless_format)]
    Ok(format!("{fetch_ui}"))
}

fn handle_writing_carta(client: &mut ClientState) -> anyhow::Result<String> {
    if !matches!(client.abyss_state.currently, AbyssMode::WritingCarta) {
        client.abyss_state.lines.clear();
    }
    client.abyss_state.currently = AbyssMode::WritingCarta;

    let mut document = Document::new();
    document
        .add_heading(HeadingLevel::H2, &client.lang.write_header)
        .add_blank_line();
    for idx in 0..=client.abyss_state.lines.len() {
        let line_number = idx + 1;
        /* Right-aligned padding for line numbers, such as:
         *  [1] lorem ispum
         * [10] hello world */
        let padding = {
            const PAD: usize = 2;
            let digit_count = {
                let mut n = line_number;
                let mut count = 1;
                while n > 10 {
                    n /= 10;
                    count += 1;
                }
                count
            };
            " ".repeat(PAD - digit_count)
        };
        document.add_link(
            WRITE_CHANGE_LINKS_LOOKUP_FROM_LINE_NUMBER[line_number],
            match line_number {
                _filled_lines if (1..=client.abyss_state.lines.len()).contains(&_filled_lines) => {
                    format!(
                        "{padding}[{line_number}] {line}",
                        line = &client.abyss_state.lines[idx]
                    )
                }
                _new_line => format!(
                    "{padding}[{line_number}] {}",
                    client.lang.write_new_line_link
                ),
            },
        );
    }
    document
        .add_blank_line()
        .add_link("help", &client.lang.write_help_link)
        .add_link("fetch", &client.lang.write_return_link);

    Ok(document.to_string())
}
fn handle_write_line(
    client: &mut ClientState,
    context: &RouteContext,
    line_number: usize,
) -> anyhow::Result<windmark::response::Response> {
    log::trace!(
        "client with id {id} is writing line on url {url:?}",
        id = client.id(),
        url = context.url
    );

    // User input
    if let Some(query) = context.url.query() {
        let query = decode(query).context("malformed uri encoding for query, expected utf-8")?;

        // Empty to cancel
        if query.trim().is_empty() {
            return Ok(windmark::response::Response::temporary_redirect(format!(
                "/{lang}/abyss/",
                lang = &client.lang.code,
            )));
        }

        // New line
        if line_number == client.abyss_state.lines.len() + 1 {
            client.abyss_state.lines.push(String::new());
        }

        let line = client
            .abyss_state
            .lines
            .get_mut(line_number - 1)
            .context("invalid line number")?;
        *line = query.to_string();

        return Ok(windmark::response::Response::temporary_redirect(format!(
            "/{lang}/abyss/",
            lang = &client.lang.code,
        )));
    }

    return Ok(windmark::response::Response::input(
        &client.lang.write_new_line_input,
    ));

    todo!()
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
    client.poke();
    if lang.is_none() {
        return Ok(windmark::response::Response::permanent_redirect(format!(
            "/{lang}/abyss",
            lang = client.lang.code
        )));
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
            "write" => client.abyss_state.currently = AbyssMode::WritingCarta,
            write_line if state.starts_with("write") => {
                let line_number = write_line.trim_start_matches("write-").parse::<usize>()?;
                let line_number = line_number
                    .checked_sub(1)
                    .context("line number underflow")?
                    + 1;
                return handle_write_line(&mut client, &context, line_number);
            }
            "help" => {
                let flash = client.lang.write_help_status.clone();
                client.abyss_state.to_flash.push(flash)
            }
            _ => (),
        };
        return Ok(windmark::response::Response::temporary_redirect(format!(
            "/{lang}/abyss/",
            lang = client.lang.code
        )));
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
