use windmark::context::RouteContext;

use crate::components::certificate::hash_certificate;

pub fn text_input(context: RouteContext) -> anyhow::Result<String> {
    Ok("none".into())
}
