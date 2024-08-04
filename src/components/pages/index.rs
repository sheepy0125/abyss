use crate::i18n::Lang;

use twinstar::{document::HeadingLevel, Document};
use windmark::context::RouteContext;

pub fn index(_context: RouteContext, lang: &'static Lang) -> anyhow::Result<String> {
    Ok(Document::new()
        .add_heading(HeadingLevel::H1, &lang.index_header)
        .add_link("abyss/", &lang.abyss_enter_link)
        .to_string())
}
