pub fn escape<S>(input: S) -> String
where
    S: Into<String>
{
    const ESCAPE: [char; 18] = [
        '_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!',
    ];
    let mut output = String::new();
    for c in input.into().chars() {
        if ESCAPE.contains(&c) {
            output.push('\\');
        }
        output.push(c);
    }
    output
}