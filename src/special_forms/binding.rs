use std::rc::Rc;

use crate::{
    environment::{Env, EnvRc},
    err,
    error::Result,
    evaluator::Evaluator,
    expression::Exp,
    special_forms::core::begin_impl,
};

pub fn register(eval: &Evaluator) {
    eval.register_special_form("let", |args, env, eval| let_impl(args, env, eval, false));
    eval.register_special_form("let*", |args, env, eval| let_impl(args, env, eval, true));
}

fn let_impl(args: &[Exp], env: &EnvRc, eval: &Evaluator, star: bool) -> Result<Exp> {
    if args.is_empty() {
        err!("let requires bindings and least one body expression")
    }

    let Exp::List(binding) = &args[0] else {
        err!("let bindings should be a list")
    };

    let child_env = Env::new_child(Rc::clone(&env));

    for pair in binding {
        match pair {
            Exp::List(list) if list.len() == 2 => {
                let Exp::Symbol(k) = &list[0] else {
                    err!("expected symbol in binding")
                };
                let v = if star {
                    eval.eval(&list[1], &child_env)
                } else {
                    eval.eval(&list[1], env)
                }?;
                let _ = &child_env.define(k, v);
            }
            _ => err!("invalid biding pair"),
        }
    }

    let body = &args[1..];
    begin_impl(body, &child_env, eval)
}
