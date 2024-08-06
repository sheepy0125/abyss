use crate::{
    database::{DatabaseCache, DATABASE, DATABASE_CACHE},
    state::ClientState,
};

use anyhow::anyhow;
use twinstar::{document::HeadingLevel, Document};
use windmark::context::RouteContext;

pub fn handle_submit_confirmation(
    client: &mut ClientState,
) -> anyhow::Result<windmark::response::Response> {
    Ok(windmark::response::Response::success(
        Document::new()
            .add_heading(HeadingLevel::H1, &client.lang.write_header)
            .add_blank_line()
            .add_link("submit", &client.lang.submit_confirmation_link)
            .add_link("write", &client.lang.cancel_link)
            .to_string(),
    ))
}

pub fn handle_submit_new(
    client: &mut ClientState,
    context: &RouteContext,
    reply_uuid: Option<String>,
) -> anyhow::Result<windmark::response::Response> {
    // Ensure carta isn't blank!!
    if client.abyss_state.write_state.lines.is_empty() {
        client
            .abyss_state
            .to_flash
            .push(client.lang.empty_carta_error.clone());
        return client.redirect_to_abyss();
    }

    let mut parent = None;
    if let Some(reply_uuid) = reply_uuid {
        let reply_carta = DatabaseCache::get_or_else(&DATABASE_CACHE.carta, &reply_uuid, &|| {
            let mut database_guard = DATABASE
                .lock()
                .map_err(|_| anyhow!("failed to lock database mutex"))?;
            database_guard.fetch_carta_uuid(&reply_uuid)
        })?;
        parent = Some(reply_carta.id);
    }

    let mut database_guard = DATABASE
        .lock()
        .map_err(|_| anyhow!("failed to lock the database"))?;

    let carta = database_guard.insert_carta(
        Some(client.id() as _),
        parent,
        std::mem::take(&mut client.abyss_state.write_state.lines).join("\n"),
        std::mem::take(&mut client.abyss_state.write_state.title),
        std::mem::take(&mut client.abyss_state.write_state.from),
        client.lang,
        context
            .peer_address
            .map(|ip| ip.ip().to_string())
            .unwrap_or("0.0.0.0".to_string()),
    )?;

    Ok(windmark::response::Response::success(
        Document::new()
            .add_heading(
                twinstar::document::HeadingLevel::H1,
                &client.lang.successful_submission_header,
            )
            .add_text(&client.lang.successful_submission_modification_text)
            .add_heading(
                HeadingLevel::H3,
                format!("{pin}{id}", id = carta.id, pin = carta.modification_code),
            )
            .add_blank_line()
            .add_link("fetch", &client.lang.return_link),
    ))
}
