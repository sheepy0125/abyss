use crate::{abyss::AbyssMode, database::DATABASE, state::ClientState};

use anyhow::{anyhow, Context as _};
use twinstar::Document;

pub fn handle_submit_confirmation(
    client: &mut ClientState,
) -> anyhow::Result<windmark::response::Response> {
    Ok(windmark::response::Response::success(
        Document::new()
            .add_link("submit", &client.lang.submit_confirmation_link)
            .add_link("write", &client.lang.cancel_link)
            .to_string(),
    ))
}

pub fn handle_submit_new(client: &mut ClientState) -> anyhow::Result<windmark::response::Response> {
    let mut database_guard = DATABASE
        .lock()
        .map_err(|_| anyhow!("failed to lock the database"))?;

    // Ensure carta isn't blank!!
    if client.abyss_state.write_state.lines.is_empty() {
        client
            .abyss_state
            .to_flash
            .push(client.lang.empty_carta_error.clone());
        return client.redirect_to_abyss();
    }

    let carta = database_guard.insert_carta(
        Some(client.id() as _),
        None,
        std::mem::take(&mut client.abyss_state.write_state.lines).join("\n"),
        std::mem::take(&mut client.abyss_state.write_state.title),
        std::mem::take(&mut client.abyss_state.write_state.from),
        client.lang,
    )?;

    Ok(windmark::response::Response::success(
        Document::new()
            .add_heading(
                twinstar::document::HeadingLevel::H2,
                &client.lang.successful_submission_header,
            )
            .add_text(&client.lang.successful_submission_pin_reminder_text)
            .add_text(format!(
                "ID: {id}; PIN: {pin}",
                id = carta.id,
                pin = carta.modification_code
            ))
            .add_blank_line()
            .add_link("fetch", &client.lang.write_return_link),
    ))
}
