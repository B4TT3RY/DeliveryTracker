pub struct Args<'a> {
    inner: &'a str,
}

pub struct Command<'a> {
    pub label: &'a str,
    pub username: Option<&'a str>,
    rest: &'a str,
}

impl<'a> Iterator for Args<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.inner.trim_start();
        if line.is_empty() {
            None
        } else if let Some((prefix, suffix)) = line.split_once(char::is_whitespace) {
            self.inner = suffix;
            Some(prefix)
        } else {
            self.inner = "";
            Some(line)
        }
    }
}

impl<'a> Command<'a> {
    pub fn new(line: &'a str) -> Self {
        let (label_with_username, rest) =
            line.split_once(char::is_whitespace).unwrap_or((line, ""));
        let (label, username) = label_with_username
            .split_once('@')
            .map(|(label, username)| (label, Some(username)))
            .unwrap_or((label_with_username, None));
        Self {
            label,
            username,
            rest,
        }
    }

    pub fn args(&self) -> Args<'a> {
        Args { inner: self.rest }
    }
}
