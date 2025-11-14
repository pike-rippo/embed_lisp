use std::io::Write;

use crate::{
    SyntaxError, environment::SharedEnv, evaluator::Evaluator, exp::Exp, flow::EvalResult,
};

pub fn register(env: &SharedEnv) {
    env.define("print", Exp::Primitive(print_impl));
    env.define("println", Exp::Primitive(println_impl));
}

fn print_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("print", 1, args.len()));
    }
    let s = match &args[0] {
        Exp::String(s) => s.clone(),
        otherwise => format!("{}", otherwise),
    };
    print!("{}", s);
    std::io::stdout().flush().unwrap();

    Ok(Exp::String(s).value_flow())
}

fn println_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.is_empty() {
        println!();
        return Ok(Exp::Nil.value_flow());
    }
    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("println", 1, args.len()));
    }
    let s = match &args[0] {
        Exp::String(s) => s.clone(),
        otherwise => format!("{}", otherwise),
    };
    println!("{}", s);

    Ok(Exp::String(s).value_flow())
}
