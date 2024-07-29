//! Abyss

use crate::database::establish_connection;

use consts::FOOTER;
use dotenvy::dotenv;
use pages::page_result_to_response;
use windmark::response::Response;

pub mod consts;
pub mod database;
pub mod pages;
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
        .mount("/", |c| page_result_to_response(pages::index::index(c)))
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
