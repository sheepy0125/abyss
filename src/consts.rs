//! Lazily loaded constants and regular constants.

use lazy_static::lazy_static;
use std::path::PathBuf;

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

pub const FOOTER: &str = r#"
```
       _
  __ _| |__  _   _ ___ ___
 / _` | '_ \| | | / __/ __|
| (_| | |_) | |_| \__ \__ \
 \__,_|_.__/ \__, |___/___/
             |___/
```
"#;
