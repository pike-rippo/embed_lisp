use std::rc::Rc;

use crate::{
    environment::EnvRc, err, error::Result, evaluator::Evaluator, expression::Exp,
    lambda::LambdaExp,
};

pub fn register(eval: &Evaluator) {
    eval.register_special_form("define-macro", define_macro_impl);
}

fn define_macro_impl(args: &[Exp], env: &EnvRc, _: &Evaluator) -> Result<Exp> {
    if args.len() != 3 {
        err!("define-macro takes 3 arguments: name, args, and body")
    }

    let name = match &args[0] {
        Exp::Symbol(s) => s,
        _ => err!("first argument to def-macro must be a symbol"),
    };

    let lambda = LambdaExp::new(Rc::new(args[1].clone()), Rc::new(args[2].clone()));
    Ok(env.define(name, Exp::Macro(lambda)))
}
