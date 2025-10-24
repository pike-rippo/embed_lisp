use std::{fmt::Display, io::Write};

use colorize::Colorize;
use embed_lisp::Interpreter;

fn read_line(depth: i32, input: &mut String) {
    // print!("{}", format!("[depth {}] > ", depth));
    if depth == 0 {
        print!("{}", format!("[depth {}]  > ", depth));
    } else {
        print!("{}", format!("[depth {}] *> ", depth));
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
    let mut input = String::new();
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
                Err(e) => println!("{}", e.bright_red()),
                Ok(val) => println!("=> {}", val.bright_green()),
            }
            buffer.clear();
        }
    }
}

// fn main() {
//     let parser = Parser::new();
//     // let env = Env::new();
//     let builtin = Env::builtin_env();
//     builtin.define(
//         "start-time",
//         Exp::String(format!("{:?}\n", std::time::SystemTime::now())),
//     );
//     let env = Env::new_with_builtin(builtin);
//     let eval = Evaluator::new();
//     let inputs = [
//         //
//         // r#"
//         // (define-macro parse-eval str
//         //     `(for (e ))
//         // )
//         // "#,
//         // (eval (parse "(define a 1) (define b 2)"))
//         // (eval (parse (call (open "test.txt" "r") "read")))
//         r#"
//         (define-macro import (filename)
//             `(eval (parse (call (open ,filename "r") "read"))))

//         (import "test.txt")

//         (dump-env)
//         "#,
//         //
//     ];
//     let exps = match parser.parse(inputs.join("")) {
//         Ok(exps) => exps,
//         Err(e) => panic!("{}", e),
//     };

//     for exp in exps {
//         let r = eval.eval(&exp, &env);
//         match r {
//             Ok(val) => println!("=> {}", val.bright_green()),
//             Err(e) => println!("{}", e.bright_red()),
//         }
//     }
// }
