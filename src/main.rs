#[cfg(feature = "async")]
use std::sync::LazyLock;

#[cfg(feature = "async")]
use tokio::runtime::Runtime;

use crate::{environment::Env, evaluator::Evaluator, expression::Exp, parser::Parser};

mod builtins;
mod environment;
mod error;
mod evaluator;
mod expression;
mod future;
mod lambda;
mod native;
mod parser;
mod replacer;
mod special_forms;
mod typedef;

#[cfg(feature = "async")]
pub static GLOBAL_RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("failed to create tokio runtime"));

fn main() {
    let parser = Parser::new();
    // let env = Env::new();
    let builtin = Env::builtin_env();
    builtin.define(
        "start-time",
        Exp::String(format!("{:?}\n", std::time::SystemTime::now())),
    );
    let env = Env::new_with_builtin(builtin);
    let eval = Evaluator::new();
    let inputs = [
        //
        // r#"(define f (open "test.txt" "a+"))"#,
        // r#"(define content (call f "read"))"#,
        // "(call f \"write\" start-time)",
        // "(for (i (range 1 10)) (if (= (% i 2) 0) (print i) (print i)))",
        // "(define x (sleep))",
        // "(await (sleep))",
        // "(await x)",
        // "x",
        // "(dump-env)",
        // "(async (for (i (range 0 10)) i))"
        // r#"
        // (define slow-add (lambda (a b)
        //     (begin
        //         (sleep)
        //         (+ a b))))
        // "#,
        // "(trace-eval)",
        r#"
        (define-macro slow-add (a b)
            `(begin
                (await (sleep))
                (+ ,a ,b)))
        "#,
        "(define f1 (async (slow-add 1 2)))",
        "(define f2 (async (slow-add 10 20)))",
        // "(define f1 (slow-add 1 2))",
        // "(define f2 (slow-add 10 20))",
        r#"(print "calc started")"#,
        "(await f1)",
        "(await f2)",
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
