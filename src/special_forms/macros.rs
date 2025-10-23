use std::rc::Rc;

use crate::{
    err,
    error::Result,
    evaluator::Evaluator,
    expression::Exp,
    lambda::LambdaExp,
    typedef::{Shared, SharedEnv},
};

pub fn register(eval: &Evaluator) {
    eval.register_special_form("define-macro", define_macro_impl);
}

fn define_macro_impl(args: &[Exp], env: &SharedEnv, _: &Evaluator) -> Result<Exp> {
    if args.len() != 3 {
        err!("define-macro takes 3 arguments: name, args, and body")
    }

    let name = match &args[0] {
        Exp::Symbol(s) => s,
        _ => err!("first argument to def-macro must be a symbol"),
    };

    let lambda = LambdaExp::new(Shared::new(args[1].clone()), Shared::new(args[2].clone()));
    Ok(env.define(name, Exp::Macro(lambda)))
}
