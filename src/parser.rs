use crate::{
    error::{Error, Result},
    expression::Exp,
    replacer::Replacer,
};

#[derive(Debug, Default)]
pub struct Parser {}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(&self, input: impl Into<String>) -> Result<Vec<Exp>> {
        let formatted = Replacer::new(input.into())
            .insert_whitespaces_outside_double_quote(&[
                ("(", None, None),
                (")", None, None),
                ("'", Some('#'), None),
                ("`", None, None),
                (",", None, Some('@')),
                (",@", None, None),
                (".", Some('.'), Some('.')),
                // ("#'", None, None),
                // ("...", None, None),
            ])
            .value();

        let input = split_whitespace_outside_quote(&formatted);

        let mut exps = Vec::new();
        let mut rest = &input[..];
        while !rest.is_empty() {
            let (exp, new_rest) = tokenize(rest)?;
            exps.push(exp);
            rest = new_rest;
        }
        Ok(exps)
    }
}

fn tokenize(input: &[String]) -> Result<(Exp, &[String])> {
    let (front, rest) = input
        .split_first()
        .ok_or(Error::from("could not get token"))?;
    match &front[..] {
        "(" => read_seq(rest),
        ")" => Err(Error::from("unexpected ')'")),
        "'" => {
            let (quoted_exp, new_rest) = tokenize(rest)?;
            Ok((
                Exp::List(vec![Exp::Symbol("quote".to_string()), quoted_exp]),
                new_rest,
            ))
        }
        "`" => {
            let (quoted_exp, new_rest) = tokenize(rest)?;
            Ok((
                Exp::List(vec![Exp::Symbol("quasiquote".to_string()), quoted_exp]),
                new_rest,
            ))
        }
        "," => {
            let (quoted_exp, new_rest) = tokenize(rest)?;
            Ok((
                Exp::List(vec![Exp::Symbol("unquote".to_string()), quoted_exp]),
                new_rest,
            ))
        }
        ",@" => {
            let (quoted_exp, new_rest) = tokenize(rest)?;
            Ok((
                Exp::List(vec![
                    Exp::Symbol("unquote-splicing".to_string()),
                    quoted_exp,
                ]),
                new_rest,
            ))
        }
        _ => Ok((parse_atomic(front), rest)),
    }
}

fn read_seq(input: &[String]) -> Result<(Exp, &[String])> {
    let mut res: Vec<Exp> = vec![];
    let mut xs = input;
    loop {
        let (next, rest) = xs.split_first().ok_or(Error::from("could not find ')'"))?;

        if next == ")" {
            return Ok((Exp::List(res), rest));
        }

        // obj.methodで、methodにリストは付け付けない
        if next == "." {
            res.insert(res.len() - 1, Exp::Symbol("call".to_string()));
            let method = rest.first().ok_or(Error::from("unexpected '.'"))?;
            res.push(Exp::String(method.to_string()));
            xs = rest.get(1..).ok_or(Error::from("unexpected '.'"))?;
            if xs.first().is_some_and(|e| e == ")") {
                return Ok((Exp::List(res), &xs[1..]));
            }
        }

        let (exp, new_xs) = tokenize(xs)?;
        res.push(exp);
        xs = new_xs;
    }
}

fn parse_atomic(input: &str) -> Exp {
    match input {
        "nil" => Exp::Nil,
        "true" => Exp::Bool(true),
        "false" => Exp::Bool(false),
        _ => {
            if input.starts_with("\"") && input.ends_with("\"") {
                return Exp::String(input[1..(input.len() - 1)].to_string());
            }
            let potential_float: std::result::Result<f64, std::num::ParseFloatError> =
                input.parse();
            match potential_float {
                Ok(v) => Exp::Number(v),
                Err(_) => Exp::Symbol(input.to_string().clone()),
            }
        }
    }
}

fn split_whitespace_outside_quote(s: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut current_token = String::new();
    let mut inside = false;
    let chars = s.chars();
    let mut pre_char_was_backslash = false;

    // while let Some(c) = chars.next() {
    for c in chars {
        if c == '"' && !pre_char_was_backslash {
            inside = !inside;
            current_token.push(c);
        } else if c.is_whitespace() && !inside {
            if !current_token.is_empty() {
                tokens.push(current_token.clone());
                current_token.clear();
            }
        } else {
            current_token.push(c);
        }

        pre_char_was_backslash = c == '\\' && !pre_char_was_backslash;
    }

    if !current_token.is_empty() {
        tokens.push(current_token);
    }
    tokens
}

#[cfg(test)]
mod test {
    use crate::{expression::Exp, parser::Parser};

    #[test]
    fn parser() {
        let p = Parser::new();
        let parsed = p.parse("(+ 1 1)".to_string());
        if let Ok(v) = parsed {
            assert_eq!(
                v,
                vec![Exp::List(vec![
                    Exp::Symbol("+".to_string()),
                    Exp::Number(1f64),
                    Exp::Number(1f64),
                ])]
            );
        } else {
            panic!("{}", parsed.unwrap_err())
        }
    }
}
