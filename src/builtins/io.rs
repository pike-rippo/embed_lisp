use crate::{evaluator::Evaluator, expression::Exp, flow::EvalResult, typedef::SharedEnv};

pub fn register(env: &SharedEnv) {
    env.define("print", Exp::Function(print_impl));
}

fn print_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    for arg in args {
        println!("{}", arg);
    }

    Ok(Exp::Bool(true).value_flow())
}
