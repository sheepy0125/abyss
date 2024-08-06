use crate::{abyss::CartaInformation, state::ClientState};

use twinstar::{document::HeadingLevel, Document};

use super::view_carta::display_field;

/// Fetch cartas page UI
pub fn handle_fetching_cartas(client: &mut ClientState) -> anyhow::Result<String> {
    let mut document = Document::new();
    document
        .add_heading(HeadingLevel::H1, &client.lang.abyss_header)
        .add_blank_line()
        .add_link("peek", &client.lang.fetch_link)
        .add_link("write", &client.lang.write_link)
        .add_blank_line();

    document.add_heading(HeadingLevel::H3, "===");
    for CartaInformation { carta, .. } in &client.abyss_state.top_level_cartas_loaded {
        document.add_link(
            format!("read-{uuid}", uuid = carta.uuid).as_str(),
            format!(
                "{from} - {title}",
                from = display_field(&carta.sender, &client.lang.from_sentinel),
                title = display_field(&carta.title, &client.lang.untitled_sentinel)
            ),
        );
    }
    document
        .add_heading(HeadingLevel::H3, "===")
        .add_blank_line()
        .add_link("..", "<--");

    Ok(document.to_string())
}
