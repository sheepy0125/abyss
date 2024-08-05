use crate::{
    database::{DatabaseCache, DATABASE, DATABASE_CACHE},
    state::ClientState,
};

use anyhow::anyhow;
use twinstar::{document::HeadingLevel, Document};

/// Fetch cartas page UI
pub fn handle_viewing_carta(client: &mut ClientState, uuid: String) -> anyhow::Result<String> {
    let mut document = Document::new();

    let carta = DatabaseCache::get_or_else(&DATABASE_CACHE.carta, &uuid, &|| {
        let mut database_guard = DATABASE
            .lock()
            .map_err(|_| anyhow!("failed to lock database mutex"))?;
        database_guard.fetch_carta_uuid(&uuid)
    })?;

    document.add_heading(
        HeadingLevel::H3,
        format!(
            "=== {from} - {title}",
            from = carta
                .sender
                .as_deref()
                .unwrap_or(&client.lang.write_from_sentinel)
                .trim_end(),
            title = carta
                .title
                .as_deref()
                .unwrap_or(&client.lang.write_untitled_sentinel)
        ),
    );

    for line in carta.content.split('\n') {
        document.add_text(line);
    }

    document.add_heading(HeadingLevel::H3, "===");

    document
        .add_blank_line()
        .add_link("fetch", &client.lang.write_return_link);

    Ok(document.to_string())
}
