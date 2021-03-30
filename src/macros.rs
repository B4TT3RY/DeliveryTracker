#[macro_export]
macro_rules! get_html_string {
    ($document:ident, $selector:expr) => {{
        let selector = Selector::parse($selector).unwrap();
        $document
            .select(&selector)
            .next()
            .unwrap()
            .text()
            .fold("".to_string(), |l, r| format!("{} {}", l, r))
            .trim()
            .to_string()
    }};
}
