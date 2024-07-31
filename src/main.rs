//! Abyss

use std::time::Duration;

use crate::abyss::handle_client_in_abyss;
use crate::consts::FOOTER;
use crate::database::establish_connection;

use components::certificate::require_certificate;
use dotenvy::dotenv;
use state::ClientState;
use tokio::spawn;

pub mod abyss;
pub mod components;
pub mod consts;
pub mod database;
pub mod schema;
pub mod state;
pub mod tree;

pub fn result_to_response(result: anyhow::Result<String>) -> windmark::response::Response {
    match result {
        Ok(res) => windmark::response::Response::success(res),
        Err(e) => windmark::response::Response::temporary_failure(format!("error! {e}")),
    }
}
pub fn windmark_response_result_to_response(
    result: anyhow::Result<windmark::response::Response>,
) -> windmark::response::Response {
    match result {
        Ok(res) => res,
        Err(e) => windmark::response::Response::temporary_failure(format!("error! {e}")),
    }
}

#[windmark::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;
    pretty_env_logger::init();

    // Periodically prune old clients
    spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await; // xxx: debug
            ClientState::prune_clients().unwrap();
        }
    });

    let abyss_handle = |context| {
        if let Err(resp) = require_certificate(&context) {
            return resp;
        };
        windmark_response_result_to_response(handle_client_in_abyss(context))
    };

    // fixme: struct routers don't work in windmark? lol
    windmark::router::Router::new()
        .set_private_key_file("server.key")
        .set_certificate_file("server.crt")
        .enable_default_logger(false)
        .set_fix_path(true)
        // index
        .mount("/", |c| {
            // page_result_to_response(components::pages::index::index(c))
            result_to_response(components::text_input::text_input(c)) // testing
        })
        // abyss
        .mount("/abyss/", abyss_handle)
        .mount("/abyss/:state", abyss_handle)
        .add_footer(|_| FOOTER.to_string())
        // route unmatched
        .set_error_handler(|_context| {
            windmark::response::Response::not_found(
                "you made a wrong turn, my friend. route not found.",
            )
        })
        .run()
        .await
        .map_err(|e| anyhow::anyhow!("router failed: {e}"))?;

    Ok(())
}
