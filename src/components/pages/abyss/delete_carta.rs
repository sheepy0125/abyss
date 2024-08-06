use crate::{abyss::CartaInformation, database::DATABASE, i18n::Lang, state::ClientState};

use anyhow::{anyhow, Context as _};
use twinstar::{document::HeadingLevel, Document};
use windmark::context::RouteContext;

/// Delete cartas page UI
pub fn handle_deleting_cartas(
    context: RouteContext,
    lang: &Lang,
) -> anyhow::Result<windmark::response::Response> {
    let state = context.parameters.get("state").map(String::as_str);

    // Code input
    if let Some("code") = state {
        let query = match context.url.query() {
            Some(query) => query.trim(),
            None => return Ok(windmark::response::Response::input(&lang.code_input)),
        };

        let (pin, id) = query
            .split_at_checked(6)
            .context("invalid access code format")?;
        let id = id.parse::<usize>().context("invalid access code format")?;

        let mut database_guard = DATABASE
            .lock()
            .map_err(|_| anyhow!("failed to lock database mutex"))?;
        let carta = database_guard.redact_carta(id as _, pin, &lang.deleted)?;

        return Ok(windmark::response::Response::temporary_redirect(
            match carta {
                Some(_) => "success",
                None => "failure",
            },
        ));
    }

    let mut document = Document::new();

    if let Some("success") = state {
        document
            .add_text(&lang.deletion_successful)
            .add_blank_line();
    } else if let Some("failure") = state {
        document.add_text(&lang.deletion_failure).add_blank_line();
    }

    document
        .add_heading(HeadingLevel::H1, &lang.delete_header)
        .add_blank_line()
        .add_text(&lang.delete_instructions_text)
        .add_link("../terms", &lang.terms_link)
        .add_link("../abyss/view", &lang.abyss_view_link)
        .add_blank_line()
        .add_heading(HeadingLevel::H3, "===")
        .add_link("code", &lang.delete_code_link)
        .add_heading(HeadingLevel::H3, "===")
        .add_blank_line()
        .add_link("..", "<--");

    Ok(windmark::response::Response::success(document.to_string()))
}
