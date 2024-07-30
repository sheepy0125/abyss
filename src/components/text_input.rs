use windmark::context::RouteContext;

use crate::components::certificate::hash_certificate;

pub fn text_input(context: RouteContext) -> anyhow::Result<String> {
    dbg!(hash_certificate(&context.certificate.unwrap()));
    Ok("none".into())
}
