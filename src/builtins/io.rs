use crate::{error::Result, evaluator::Evaluator, expression::Exp, ok, typedef::SharedEnv};

pub fn register(env: &SharedEnv) {
    env.define("print", Exp::Function(print_impl));
}

fn print_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> Result<Exp> {
    for arg in args {
        println!("{}", arg);
    }

    ok!(true)
}
