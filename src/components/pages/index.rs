use crate::i18n::Lang;

use twinstar::{document::HeadingLevel, Document};
use windmark::context::RouteContext;

pub fn index(_context: RouteContext, lang: &'static Lang) -> anyhow::Result<String> {
    Ok(Document::new()
        .add_heading(HeadingLevel::H1, &lang.index_header)
        .add_blank_line()
        .add_heading(HeadingLevel::H2, &lang.index_about_header)
        .add_text(&lang.index_about_text)
        .add_blank_line()
        .add_heading(HeadingLevel::H2, &lang.index_ready_header)
        .add_text(&lang.index_cert_text)
        .add_link("terms/", &lang.abyss_terms_link)
        .add_link("abyss/fetch", &lang.abyss_enter_link)
        .add_blank_line()
        .add_heading(HeadingLevel::H2, &lang.index_management_header)
        .add_link("abyss/view", &lang.abyss_view_link)
        .add_link("delete/", &lang.abyss_delete_link)
        .add_blank_line()
        .add_link(
            "https://github.com/sheepy0125/abyss",
            &lang.source_code_link,
        )
        .to_string())
}
