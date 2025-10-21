use crate::{environment::EnvRc, error::Result, evaluator::Evaluator, expression::Exp, ok};

pub fn register(env: &EnvRc) {
    env.define("print", Exp::Function(print_impl));
}

fn print_impl(args: &[Exp], _: &EnvRc, _: &Evaluator) -> Result<Exp> {
    for arg in args {
        println!("{}", arg);
    }

    ok!(true)
}
