use crate::{components::certificate::CERT_HASH_LEN, i18n::Lang};

use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};
use twinstar::{document::HeadingLevel, Document};
use windmark::context::RouteContext;

pub fn certless(_context: RouteContext, lang: &'static Lang) -> anyhow::Result<String> {
    let random = Alphanumeric.sample_string(&mut thread_rng(), CERT_HASH_LEN);

    Ok(Document::new()
        .add_heading(HeadingLevel::H1, &lang.certless_header)
        .add_blank_line()
        .add_text(&lang.certless_warning_text)
        .add_blank_line()
        .add_link(
            format!("{random}/abyss/").as_str(),
            &lang.certless_proceed_link,
        )
        .to_string())
}
