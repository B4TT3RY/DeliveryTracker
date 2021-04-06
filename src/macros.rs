#[macro_export]
macro_rules! get_html_string {
    ($document:ident, $selector:expr) => {{
        let result = $document
            .select(&Selector::parse($selector).unwrap())
            .flat_map(|el| el.text())
            .collect::<String>();
        Regex::new(r#"(\s{2,}|\n|\t)"#)
            .unwrap()
            .replace_all(&result, " ")
            .trim()
            .to_string()
    }};
}
