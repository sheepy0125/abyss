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
