use crate::{abyss::MAX_NUM_LINES, state::ClientState};

use lazy_static::lazy_static;
use twinstar::{document::HeadingLevel, Document};

use super::view_carta::display_field;

lazy_static! {
    // Twinstar's [`twinstar::Document`] type only allows URIs added through
    // [`Document::add_link`] to have a 'static lifetime.
    pub static ref WRITE_CHANGE_LINKS_LOOKUP_FROM_LINE_NUMBER: [&'static str; MAX_NUM_LINES + 1] = {
        std::array::from_fn(|n| format!("write-{n}").leak() as &'static str)
    };
}

// Write carta page UI
pub fn handle_writing_carta(
    client: &mut ClientState,
    reply_uuid: Option<String>,
) -> anyhow::Result<String> {
    let mut document = Document::new();
    document
        .add_heading(HeadingLevel::H1, &client.lang.write_header)
        .add_blank_line()
        .add_heading(HeadingLevel::H3, &client.lang.write_body_header);
    for idx in 0..((client.abyss_state.write_state.lines.len() + 1).min(MAX_NUM_LINES)) {
        let line_number = idx + 1;
        /* Right-aligned padding for line numbers, such as:
         * [ 1] lorem ispum
         * [10] hello world */
        let padding = {
            let pad: usize = 1 + (client.abyss_state.write_state.lines.len() >= 10) as usize;
            let digit_count = 1 + (line_number >= 10) as usize;
            " ".repeat(pad - digit_count)
        };
        let line_number_formatted = if !client.abyss_state.write_state.hide_line_numbers {
            &format!("[{padding}{line_number}]")
        } else {
            ""
        };
        document.add_link(
            WRITE_CHANGE_LINKS_LOOKUP_FROM_LINE_NUMBER[line_number],
            match line_number {
                _filled_lines
                    if (1..=client.abyss_state.write_state.lines.len())
                        .contains(&_filled_lines) =>
                {
                    format!(
                        "{line_number_formatted} {line}",
                        line = &client.abyss_state.write_state.lines[idx]
                    )
                }
                _new_line => format!(
                    "{line_number_formatted} {}",
                    client.lang.write_new_line_link
                ),
            },
        );
    }
    document
        .add_heading(HeadingLevel::H3, "===")
        .add_blank_line()
        .add_heading(HeadingLevel::H3, &client.lang.write_head_header);
    document.add_link(
        "title",
        format!(
            "{title_text}: {title}",
            title_text = &client.lang.write_title_link,
            title = display_field(
                &client.abyss_state.write_state.title,
                &client.lang.untitled_sentinel
            ),
        ),
    );
    document.add_link(
        "from",
        format!(
            "{from_text}: {from}",
            from_text = &client.lang.write_from_link,
            from = display_field(
                &client.abyss_state.write_state.from,
                &client.lang.from_sentinel
            ),
        ),
    );
    document.add_heading(HeadingLevel::H3, "===");
    document
        .add_blank_line()
        .add_link("submit-confirmation", &client.lang.write_submit_link)
        .add_link("help", &client.lang.write_help_link);
    document.add_link(
        "toggle-line-numbers",
        if !client.abyss_state.write_state.hide_line_numbers {
            &client.lang.write_hide_line_numbers_link
        } else {
            &client.lang.write_show_line_numbers_link
        },
    );
    document.add_blank_line();
    if let Some(reply_uuid) = reply_uuid {
        document.add_link(format!("read-{reply_uuid}").as_str(), "<--");
    }
    document.add_link("fetch", &client.lang.return_link);

    Ok(document.to_string())
}
