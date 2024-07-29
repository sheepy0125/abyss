use twinstar::{document::HeadingLevel, Document};
use windmark::context::RouteContext;

pub fn index(context: RouteContext) -> anyhow::Result<String> {
    Ok(Document::new()
        .add_heading(HeadingLevel::H1, "into the Abyss™ again")
        .to_string())
}
