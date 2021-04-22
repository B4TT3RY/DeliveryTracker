pub static REGEX: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(|| regex::Regex::new(r#"(\s{2,}|\n|\t)"#).unwrap());

#[macro_export]
macro_rules! get_html_string {
    ($document:ident, $selector:expr) => {{
        let result = $document.select($selector).text();
        crate::macros::REGEX
            .replace_all(&result, " ")
            .trim()
            .to_string()
    }};
}
