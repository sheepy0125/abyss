//! Abyss

use crate::abyss::handle_client_in_abyss;
use crate::consts::FOOTER;
use crate::database::establish_connection;

use components::certificate::require_certificate;
use dotenvy::dotenv;

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

#[windmark::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;
    pretty_env_logger::init();

    let database_connection = establish_connection()?;

    // todo: task to periodically prune old clients

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
        .mount("/abyss", |context| {
            if let Err(resp) = require_certificate(&context) {
                return resp;
            };
            result_to_response(handle_client_in_abyss(context))
        })
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
