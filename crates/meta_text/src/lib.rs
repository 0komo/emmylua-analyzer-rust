use i18n_backend::I18nBackend;

mod i18n_backend;

#[macro_use]
extern crate rust_i18n;

rust_i18n::i18n!("./locales", fallback = "en", backend = I18nBackend::new());

pub fn set_locale(locale: &str) {
    rust_i18n::set_locale(locale);
}

pub fn meta_keyword(key: &str) -> String {
    t!(format!("keywords.{}", key)).to_string()
}

pub fn meta_builtin_std(key: &str) -> String {
    t!(format!("builtin_std.{}", key)).to_string()
}

pub fn meta_doc_tag(key: &str) -> String {
    t!(format!("tags.{}", key)).to_string()
}

