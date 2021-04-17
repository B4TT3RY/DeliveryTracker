use once_cell::sync::OnceCell;
use regex::Regex;

pub static REGEX: OnceCell<Regex> = OnceCell::new();

#[macro_export]
macro_rules! get_html_string {
    ($document:ident, $selector:expr) => {{
        let result = $document.select($selector).text();
        crate::macros::REGEX
            .get_or_init(|| regex::Regex::new(r#"(\s{2,}|\n|\t)"#).unwrap())
            .replace_all(&result, " ")
            .trim()
            .to_string()
    }};
}
