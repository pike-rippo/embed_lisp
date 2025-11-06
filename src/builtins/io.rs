use crate::{environment::SharedEnv, evaluator::Evaluator, exp::Exp, flow::EvalResult};

pub fn register(env: &SharedEnv) {
    env.define("print", Exp::Function(print_impl));
}

fn print_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    for arg in args {
        println!("{}", arg);
    }

    Ok(Exp::Bool(true).value_flow())
}
