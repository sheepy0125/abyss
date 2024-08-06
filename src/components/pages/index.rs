use crate::i18n::Lang;

use twinstar::{document::HeadingLevel, Document};
use windmark::context::RouteContext;

pub fn index(_context: RouteContext, lang: &'static Lang) -> anyhow::Result<String> {
    Ok(Document::new()
        .add_heading(HeadingLevel::H1, &lang.index_header)
        .add_blank_line()
        .add_link("abyss/", &lang.abyss_enter_link)
        .add_link("terms/", &lang.abyss_terms_link)
        .add_blank_line()
        .add_heading(HeadingLevel::H2, &lang.index_about_header)
        .add_text(&lang.index_about_text)
        .to_string())
}
