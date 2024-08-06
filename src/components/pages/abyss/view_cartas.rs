use super::view_carta::{display_field, display_unix_timestamp};
use crate::{database::DATABASE, state::ClientState};

use anyhow::anyhow;
use twinstar::{document::HeadingLevel, Document};

/// Handle viewing cartas
pub fn handle_viewing_cartas(client: &mut ClientState) -> anyhow::Result<String> {
    let mut document = Document::new();
    document
        .add_heading(HeadingLevel::H1, &client.lang.all_header)
        .add_blank_line()
        .add_heading(HeadingLevel::H3, "===");

    let mut database_guard = DATABASE
        .lock()
        .map_err(|_| anyhow!("failed to lock database mutex"))?;
    let cartas = database_guard.fetch_cartas(client.id() as _)?;

    for carta in &cartas {
        document.add_link(
            format!("read-{uuid}", uuid = carta.uuid).as_str(),
            format!(
                "{time} / {from} - {title}",
                time = display_unix_timestamp(carta.creation as _),
                from = display_field(&carta.sender, &client.lang.from_sentinel),
                title = display_field(&carta.title, &client.lang.untitled_sentinel),
            ),
        );
    }
    if cartas.is_empty() {
        document.add_text(&client.lang.all_empty_text);
    }

    document
        .add_heading(HeadingLevel::H3, "===")
        .add_blank_line()
        .add_link("fetch", "<--");

    Ok(document.to_string())
}
