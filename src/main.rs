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
mod task;
mod typedef;

#[cfg(feature = "async")]
pub static GLOBAL_RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("failed to create tokio runtime"));

// eval
// read

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
        "(define map (create-hash-map))",
        r#"(call map "insert" "a" "apple")"#,
        r#"(call map "insert" "b" "banana")"#,
        // r#"(call map "insert" "c" "city")"#,
        r#"(map . insert "c" "city")"#,
        r#"(await (spawn (async (call map "insert" "d" "derive"))))"#,
        r#"(call map "insert" "e" "each")"#,
        r#"(map.insert "e" "each")"#,
        r#"
        (for (key (call map "keys"))
            (print (call map "get" key)))
        "#,
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
