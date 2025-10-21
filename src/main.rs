use crate::{environment::Env, evaluator::Evaluator, parser::Parser};

mod builtins;
mod environment;
mod error;
mod evaluator;
mod expression;
mod lambda;
mod native;
mod parser;
mod replacer;
mod special_forms;

fn main() {
    let parser = Parser::new();
    let env = Env::new();
    let eval = Evaluator::new();
    let inputs = [
        //
        // r#"(define f (open "test.txt" "a+"))"#,
        // r#"(define content (call f "read"))"#,
        // "(call f \"write\" \"追記\n\")",
        "(for (i (range 1 10)) (if (= (% i 2) 0) (print i) (print i)))",
    ];
    let exps = match parser.parse(inputs.join("")) {
        Ok(exps) => exps,
        Err(e) => panic!("{}", e),
    };

    for exp in exps {
        let r = eval.eval(&exp, &env);
        match r {
            Ok(val) => println!("=>{}", val),
            Err(e) => println!("{}", e),
        }
    }
}
