//! Abyss

#![allow(incomplete_features)]
#![feature(inherent_associated_types)]

use crate::abyss::handle_client_in_abyss;
use crate::consts::FOOTER;
use crate::i18n::{lookup_lang_from_code, Lang};

use components::certificate::require_certificate;
use dotenvy::dotenv;
use i18n::ensure_lazily_loaded_languages_work;
use state::ClientState;
use std::time::Duration;
use tokio::spawn;
use windmark::context::RouteContext;

pub mod abyss;
pub mod components;
pub mod consts;
pub mod database;
pub mod i18n;
pub mod schema;
pub mod state;
pub mod tree;

pub fn result_to_response(result: anyhow::Result<String>) -> windmark::response::Response {
    match result {
        Ok(res) => windmark::response::Response::success(res),
        Err(e) => {
            log::error!("{e:#?}");
            windmark::response::Response::temporary_failure(format!("error! {e}"))
        }
    }
}
pub fn windmark_response_result_to_response(
    result: anyhow::Result<windmark::response::Response>,
) -> windmark::response::Response {
    match result {
        Ok(res) => res,
        Err(e) => {
            log::error!("{e:#?}");
            windmark::response::Response::temporary_failure(format!("error! {e}"))
        }
    }
}
pub fn get_lang(context: &RouteContext) -> Option<&'static Lang> {
    context
        .parameters
        .get("lang")
        .and_then(|str_ref| lookup_lang_from_code(str_ref))
}

macro_rules! lang {
    ($context:expr) => {
        match get_lang(&$context) {
            Some(lang) => lang,
            None => return windmark::response::Response::temporary_redirect("/en/"),
        }
    };
}

#[windmark::main]
async fn main() -> anyhow::Result<()> {
    ensure_lazily_loaded_languages_work();
    dotenv()?;
    pretty_env_logger::init();

    // Periodically prune old clients
    spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await; // xxx: debug
            ClientState::prune_clients().unwrap();
        }
    });

    let index_handle = |context| {
        let lang = lang!(context);
        result_to_response(components::pages::index::index(context, lang))
    };
    let terms_handle = |context| {
        let lang = lang!(context);
        result_to_response(components::pages::terms::terms(context, lang))
    };
    let abyss_handle = |context| {
        let lang = lang!(context);
        if let Err(resp) = require_certificate(&context, lang) {
            return resp;
        };
        windmark_response_result_to_response(handle_client_in_abyss(context))
    };
    let delete_handle = |context| {
        let lang = lang!(context);
        windmark_response_result_to_response(
            components::pages::abyss::delete_carta::handle_deleting_cartas(context, lang),
        )
    };

    // Fix a link to add a trailing slash
    let fix = |context: RouteContext| {
        windmark::response::Response::temporary_redirect(format!("{}/", context.url))
    };
    // Fix a link to remove a trailing slash
    let rev_fix = |context: RouteContext| {
        windmark::response::Response::temporary_redirect(
            context.url.to_string().trim_end_matches('/'),
        )
    };

    windmark::router::Router::new()
        .set_private_key_file("server.key")
        .set_certificate_file("server.crt")
        .enable_default_logger(false)
        .set_fix_path(false)
        // index
        .mount("/", index_handle)
        .mount("/:lang", fix)
        .mount("/:lang/", index_handle)
        // terms
        .mount("/:lang/terms", fix)
        .mount("/:lang/terms/", terms_handle)
        // abyss
        .mount("/:lang/abyss", fix)
        .mount("/:lang/abyss/", abyss_handle)
        .mount("/:lang/abyss/:state", abyss_handle)
        .mount("/:lang/abyss/:state/", rev_fix)
        // delete
        .mount("/:lang/delete", fix)
        .mount("/:lang/delete/", delete_handle)
        .mount("/:lang/delete/:state", delete_handle)
        .mount("/:lang/delete/:state/", rev_fix)
        .add_footer(|_| FOOTER.to_string())
        // route unmatched
        .set_error_handler(|_context| {
            windmark::response::Response::temporary_failure("route unmatched")
        })
        .run()
        .await
        .map_err(|e| anyhow::anyhow!("router failed: {e}"))?;

    Ok(())
}
