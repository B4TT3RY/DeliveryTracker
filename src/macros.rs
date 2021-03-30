#[macro_export]
macro_rules! get_html_string {
    ($document:ident, $selector:expr) => {{
        let selector = Selector::parse($selector).unwrap();
        $document
            .select(&selector)
            .next()
            .unwrap()
            .text()
            .collect::<String>()
            .trim()
            .to_string()
    }};
}
