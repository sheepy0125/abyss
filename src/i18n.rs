use crate::consts::I18N_DIR;

use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Lang {
    /// 2-character code, e.g. "en"
    pub code: String,
    pub cert_required: String,
    /* Carta */
    pub untitled_sentinel: String,
    pub from_sentinel: String,
    /* Index page */
    pub index_header: String,
    pub index_about_header: String,
    pub index_about_text: String,
    pub abyss_enter_link: String,
    pub abyss_terms_link: String,
    pub abyss_view_link: String,
    pub abyss_delete_link: String,
    /* Terms page */
    pub terms_link: String,
    pub tos_header: String,
    pub rules_header: String,
    pub rules_preface: String,
    pub rule_1: String,
    pub rule_2: String,
    pub rule_3: String,
    pub terms_header: String,
    pub term_1: String,
    pub term_2: String,
    pub term_3: String,
    pub data_header: String,
    pub data_preface: String,
    pub data_1: String,
    pub data_2: String,
    pub data_3: String,
    pub data_4: String,
    /* Fetch page */
    pub abyss_header: String,
    pub fetch_link: String,
    pub write_link: String,
    pub return_link: String,
    pub no_new_cartas_status: String,
    /* Submit confirmation page */
    pub cancel_link: String,
    pub submit_confirmation_link: String,
    /* Submit page */
    pub empty_carta_error: String,
    pub successful_submission_header: String,
    pub successful_submission_modification_text: String,
    /* Write page */
    pub write_help_flash: String,
    pub write_header: String,
    pub write_body_header: String,
    pub write_new_line_message: String,
    pub write_new_line_link: String,
    pub write_head_header: String,
    pub write_title_link: String,
    pub write_from_link: String,
    pub write_new_field_message: String,
    pub write_submit_link: String,
    pub write_help_link: String,
    pub write_delete_command: String,
    pub write_hide_line_numbers_link: String,
    pub write_show_line_numbers_link: String,
    pub write_too_long: String,
    /* View page */
    pub view_header: String,
    pub view_replies_header: String,
    pub view_add_reply_link: String,
    pub view_report_link: String,
    pub report_submitted_flash: String,
    pub delete_code_text: String,
    /* View cartas page */
    pub all_header: String,
    pub all_empty_text: String,
    /* Delete cartas page */
    pub delete_header: String,
    pub delete_instructions_text: String,
    pub delete_code_link: String,
    pub code_input: String,
    pub deleted: String,
    pub removed: String,
    pub deletion_failure: String,
    pub deletion_successful: String,
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
