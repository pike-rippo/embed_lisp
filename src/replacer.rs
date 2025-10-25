#[derive(Debug, Default)]
pub struct Replacer {
    value: String,
}

impl Replacer {
    pub fn new(value: String) -> Self {
        Self { value }
    }

    pub fn value(self) -> String {
        self.value
    }

    pub fn insert_whitespaces_outside_double_quote(
        mut self,
        args: &[(&str, Option<char>, Option<char>)],
    ) -> Self {
        for (target, negative_lookbehind, negative_lookahead) in args {
            self = self.insert_whitespace_outside_double_quote(
                target,
                *negative_lookbehind,
                *negative_lookahead,
            );
        }
        self
    }

    pub fn insert_whitespace_outside_double_quote(
        self,
        target: &str,
        negative_lookbehind: Option<char>,
        negative_lookahead: Option<char>,
    ) -> Self {
        self.replace_outside_double_quote(
            target,
            &format!(" {} ", target),
            negative_lookbehind,
            negative_lookahead,
        )
    }

    pub fn replace_outside_double_quote(
        mut self,
        from: &str,
        to: &str,
        negative_lookbehind: Option<char>,
        negative_lookahead: Option<char>,
    ) -> Self {
        let mut result = String::with_capacity(self.value.len());
        let mut inside = false;
        let mut last: Option<char> = None;
        let mut chars = self.value.chars().peekable();
        let mut prev_char_was_backslash = false;
        #[allow(unused_assignments)]
        let mut matched = false;

        while let Some(c) = chars.next() {
            if c == '"' && !prev_char_was_backslash {
                inside = !inside;
                last = Some(c);
                result.push(c);
            } else if !inside {
                matched = false;
                if !from.is_empty()
                    && c == from.chars().next().unwrap_or('\0')
                    && (negative_lookbehind.is_none() || last != negative_lookbehind)
                    && (negative_lookahead.is_none()
                        || chars
                            .peek()
                            .is_none_or(|next| Some(*next) != negative_lookahead))
                {
                    let lookahead_str: String =
                        chars.clone().take(from.len() - 1).collect::<String>();
                    if lookahead_str == from[1..] {
                        matched = true;
                    }
                }

                if matched {
                    last = to.chars().last();
                    result.push_str(to);
                    for _ in 0..(from.len() - 1) {
                        chars.next();
                    }
                } else {
                    last = Some(c);
                    result.push(c);
                }
            } else {
                last = Some(c);
                result.push(c);
            }

            prev_char_was_backslash = c == '\\' && !prev_char_was_backslash;
        }

        self.value = result;
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn replacer() {
        let s = "(+ 1 1)";
        let r = Replacer::new(s.to_string());
        let formatted = r
            .insert_whitespaces_outside_double_quote(&[
                //
                ("(", None, None),
                (")", None, None),
            ])
            .value();

        assert_eq!(formatted, s.replace("(", " ( ").replace(")", " ) "))
    }
}
