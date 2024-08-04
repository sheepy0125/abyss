use crate::state::ClientState;

use twinstar::{document::HeadingLevel, Document};

/// Fetch cartas page UI
pub fn handle_fetching_cartas(client: &mut ClientState) -> anyhow::Result<String> {
    let abyss_state = &mut client.abyss_state;

    // Fetch UI
    let fetch_ui = Document::new()
        .add_heading(HeadingLevel::H2, &client.lang.fetch_header)
        .add_link("peek", &client.lang.fetch_link)
        .add_link("write", &client.lang.write_link)
        .to_string();

    // todo: list fetched cartas

    #[allow(clippy::useless_format)]
    Ok(format!("{fetch_ui}"))
}
