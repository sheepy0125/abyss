use crate::{abyss::CartaInformation, state::ClientState};

use twinstar::{document::HeadingLevel, Document};

/// Fetch cartas page UI
pub fn handle_fetching_cartas(client: &mut ClientState) -> anyhow::Result<String> {
    let mut document = Document::new();
    document
        .add_heading(HeadingLevel::H2, &client.lang.fetch_header)
        .add_link("peek", &client.lang.fetch_link)
        .add_link("write", &client.lang.write_link)
        .add_blank_line();

    for (
        idx,
        CartaInformation {
            title, from, uuid, ..
        },
    ) in client
        .abyss_state
        .top_level_cartas_loaded
        .iter()
        .enumerate()
    {
        document.add_link(format!("read-{uuid}").as_str(), format!("{from} - {title}"));
        if idx >= 10 {
            break;
        }
    }

    Ok(document.to_string())
}
