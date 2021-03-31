#[macro_export]
macro_rules! get_html_string {
    ($document:ident, $selector:expr) => {{
        let selector = Selector::parse($selector).unwrap();
        $document
            .select(&selector)
            .next()
            .unwrap()
            .text()
            .map(|str| str.trim())
            .filter(|str| !str.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .to_string()
    }};
}
