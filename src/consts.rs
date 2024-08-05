//! Lazily loaded constants and regular constants.

use lazy_static::lazy_static;
use std::{path::PathBuf, sync::Arc};

use crate::database::{Carta, DATABASE};

macro_rules! from_environment {
    ($key:expr) => {
        std::env::var($key).expect(&format!("not set environment variable: {}", $key))
    };
}

lazy_static! {
    pub static ref DATABASE_URL: String = from_environment!("DATABASE_URL");
    pub static ref I18N_DIR: PathBuf = std::env::current_dir()
        .unwrap()
        .join("i18n")
        .canonicalize()
        .expect("i18n directory not found");
}

lazy_static! {
    pub static ref DEFAULT_CARTA: Arc<Carta> = {
        let mut database_guard = DATABASE.lock().unwrap();
        Arc::new(database_guard.fetch_carta(0).unwrap())
    };
}

pub const FOOTER: &str = r#"
"#;
