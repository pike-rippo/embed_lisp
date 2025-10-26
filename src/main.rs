use std::io::Write;

use colorize::Colorize;
use embed_lisp::Interpreter;

fn read_line(depth: i32, input: &mut String) {
    if depth == 0 {
        print!("[depth {}]  > ", depth);
    } else {
        print!("[depth {}] *> ", depth);
    }
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(input).unwrap();
}

fn parens_balanced(s: &str) -> (bool, i32) {
    let mut depth = 0;
    let mut in_string = false;

    for c in s.chars() {
        match c {
            '"' => in_string = !in_string,
            '(' if !in_string => depth += 1,
            ')' if !in_string => {
                if depth == 0 {
                    return (false, depth);
                }
                depth -= 1;
            }
            _ => {}
        }
    }

    (depth == 0 && !in_string, depth)
}

fn main() {
    let interpreter = Interpreter::new();
    let mut buffer = String::new();
    let mut input: String;
    loop {
        let (_, depth) = parens_balanced(&buffer);
        input = "".to_string();
        read_line(depth, &mut input);
        if input.eq_ignore_ascii_case(".exit\r\n") || input.is_empty() {
            println!("{}", "Bey".yellow());
            return;
        }
        if input.eq_ignore_ascii_case(".break\r\n") {
            buffer.clear();
            continue;
        }
        buffer.push_str(&input);
        buffer.push('\n');

        let (balanced, depth) = parens_balanced(&buffer);
        if !balanced && depth == 0 {
            println!("{}", "error: too many ')'".red());
            buffer.clear();
            continue;
        }

        if balanced {
            match interpreter.eval_str(&buffer) {
                Err(e) => println!("{}\n", e.bright_red()),
                Ok(val) => println!("=> {}\n", val.bright_green()),
            }
            buffer.clear();
        }
    }
}
