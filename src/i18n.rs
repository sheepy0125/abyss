use crate::consts::I18N_DIR;

use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Lang {
    pub code: String,
    //
    pub cert_required: String,
    // index
    pub index_header: String,
    // abyss
    pub fetch: String,
    pub fetch_header: String,
    pub no_new_cartas: String,
}

// Dynamic parsing: Load file at runtime with runtime errors
macro_rules! parse_i18n_file_dynamic {
    ($lang:expr) => {{
        let buf = std::fs::read_to_string(&I18N_DIR.join(format!("{}.ron", $lang)))
            .expect(&format!("language file not found for {}", $lang));
        ron::from_str(&buf).expect(&format!("failed to parse i18n file for {}", $lang))
    }};
}

lazy_static! {
    pub static ref ENGLISH: Lang = parse_i18n_file_dynamic!("en");
}

pub fn lookup_lang_from_code(code: &str) -> Option<&'static Lang> {
    match code {
        "en" => Some(&ENGLISH),
        _ => None,
    }
}
