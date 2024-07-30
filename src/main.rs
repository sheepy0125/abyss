//! Abyss

use crate::components::pages::page_result_to_response;
use crate::consts::FOOTER;
use crate::database::establish_connection;

use dotenvy::dotenv;

pub mod components;
pub mod consts;
pub mod database;
pub mod schema;
pub mod tree;

#[windmark::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;

    let connection = establish_connection()?;

    windmark::router::Router::new()
        .set_private_key_file("server.key")
        .set_certificate_file("server.crt")
        .enable_default_logger(true)
        .set_fix_path(true)
        .mount("/", |c| {
            page_result_to_response(components::pages::index::index(c))
        })
        .add_footer(|_| FOOTER.to_string())
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
