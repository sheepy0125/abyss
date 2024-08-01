use crate::{
    components::certificate::hash_certificate,
    database::DATABASE,
    get_lang,
    i18n::{Lang, ENGLISH},
    state::ClientState,
};

use anyhow::{anyhow, Context};
use lazy_static::lazy_static;
use std::{borrow::Cow, collections::VecDeque, mem::MaybeUninit};
use twinstar::{document::HeadingLevel, Document, URIReference};
use windmark::context::RouteContext;

pub const MAX_LINE_LEN: usize = 256;
pub const MAX_NUM_LINES: usize = 50;

lazy_static! {
    // Twinstar's [`twinstar::Document`] type only allows URIs added through
    // [`Document::add_link`] to have a 'static lifetime.
    pub static ref WRITE_CHANGE_LINKS_LOOKUP_FROM_LINE_NUMBER: Vec<&'static mut &'static str> = {
        // xxx: what the fuck?
        let mut buf = Box::new(MaybeUninit::uninit_array::<MAX_NUM_LINES>());
        buf.iter_mut().enumerate().for_each(|(idx, ele)| *ele = MaybeUninit::new(format!("write-{idx}")));
        let buf = Box::leak(buf);
        unsafe {
            buf.iter().map(|init| {
                let len = init.assume_init_ref().len();
                Box::leak(Box::new(std::str::from_raw_parts(init.assume_init_ref().as_ptr(), len)))
            }).collect()
        }
    };
}

#[derive(Default)]
pub struct AbyssState {
    pub top_level_cartas_loaded: VecDeque<(String, i32)>,
    pub currently: AbyssMode,
    pub to_flash: Option<String>,
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
            client.abyss_state.to_flash = Some(client.lang.no_new_cartas_status.clone());
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
    document.add_heading(HeadingLevel::H2, &client.lang.write_header);
    document.add_link("fetch", &client.lang.write_return);
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
            *WRITE_CHANGE_LINKS_LOOKUP_FROM_LINE_NUMBER[line_number],
            match line_number {
                _filled_lines if (0..client.abyss_state.lines.len()).contains(&_filled_lines) => {
                    format!(
                        "{padding}[{line_number}] {line}",
                        line = &client.abyss_state.lines[idx]
                    )
                }
                _new_line => format!("{padding}[{line_number}] {}", client.lang.write_new_line),
            },
        );
    }

    Ok(document.to_string())
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
    if let Some(res) = context
        .parameters
        .get("state")
        .map(|str_ref| str_ref.as_str())
        .map(|state| -> anyhow::Result<()> {
            client.abyss_state.currently = match state {
                "fetch" => AbyssMode::FetchingCartas,
                "write" => AbyssMode::WritingCarta,
                "peek" => handle_peek_state_change(&mut client)?,
                _ => client.abyss_state.currently,
            };
            Ok(())
        })
    {
        res.context("handling state change")?;
        return Ok(windmark::response::Response::temporary_redirect(format!(
            "/{lang}/abyss/",
            lang = client.lang.code
        )));
    }

    let flash = match client.abyss_state.to_flash {
        Some(ref to_flash) => Document::new()
            .add_heading(HeadingLevel::H3, to_flash)
            .add_blank_line()
            .to_string(),
        None => "".to_string(),
    };
    client.abyss_state.to_flash = None;

    let body = match client.abyss_state.currently {
        AbyssMode::FetchingCartas => handle_fetching_cartas(&mut client)?,
        AbyssMode::WritingCarta => handle_writing_carta(&mut client)?,
        AbyssMode::ViewingCarta(id) => todo!("viewing carta"),
    };
    Ok(windmark::response::Response::success(format!(
        "{flash}{body}"
    )))
}
