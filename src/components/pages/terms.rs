use crate::i18n::Lang;

use twinstar::{document::HeadingLevel, Document};
use windmark::context::RouteContext;

pub fn terms(_context: RouteContext, lang: &'static Lang) -> anyhow::Result<String> {
    Ok(Document::new()
        .add_heading(HeadingLevel::H1, &lang.tos_header)
        .add_blank_line()
        .add_heading(HeadingLevel::H2, &lang.rules_header)
        .add_text(&lang.rules_preface)
        .add_unordered_list_item(&lang.rule_1)
        .add_unordered_list_item(&lang.rule_2)
        .add_unordered_list_item(&lang.rule_3)
        .add_blank_line()
        .add_heading(HeadingLevel::H2, &lang.terms_header)
        .add_unordered_list_item(&lang.term_1)
        .add_unordered_list_item(&lang.term_2)
        .add_unordered_list_item(&lang.term_3)
        .add_blank_line()
        .add_heading(HeadingLevel::H2, &lang.data_header)
        .add_text(&lang.data_preface)
        .add_unordered_list_item(&lang.data_1)
        .add_unordered_list_item(&lang.data_2)
        .add_unordered_list_item(&lang.data_3)
        .add_unordered_list_item(&lang.data_4)
        .add_blank_line()
        .add_link("..", "<--")
        .to_string())
}
