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
    pub abyss_enter_link: String,
    // abyss
    pub fetch_header: String,
    pub fetch_link: String,
    pub no_new_cartas_status: String,
    pub cancel_link: String,
    pub submit_confirmation_link: String,
    pub empty_carta_error: String,
    pub write_header: String,
    pub write_body_header: String,
    pub write_head_header: String,
    pub write_title_header: String,
    pub write_from_header: String,
    pub write_link: String,
    pub write_submit_link: String,
    pub write_help_link: String,
    pub write_help_status: String,
    pub write_return_link: String,
    pub write_new_line_link: String,
    pub write_untitled_sentinel: String,
    pub write_from_sentinel: String,
    pub write_new_field: String,
    pub write_new_line_input: String,
    pub write_delete_command: String,
    pub write_hide_line_numbers_link: String,
    pub write_show_line_numbers_link: String,
    pub write_too_long: String,
    pub successful_submission_header: String,
    pub successful_submission_pin_reminder_text: String,
    pub view_replies_header: String,
    pub view_add_reply_link: String,
    pub report_submitted_flash: String,
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

pub fn ensure_lazily_loaded_languages_work() {
    assert!("en" == &ENGLISH.code);
}
