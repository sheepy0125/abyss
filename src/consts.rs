/// Lazily loaded constants and regular constants.
use lazy_static::lazy_static;

macro_rules! from_environment {
    ($key:expr) => {
        std::env::var($key).expect(&format!("not set environment variable: {}", $key))
    };
}

lazy_static! {
    pub static ref DATABASE_URL: String = from_environment!("DATABASE_URL");
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
