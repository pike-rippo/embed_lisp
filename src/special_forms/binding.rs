use crate::{
    environment::{Env, SharedEnv},
    error::SyntaxError,
    evaluator::Evaluator,
    expression::Exp,
    flow::EvalResult,
    special_forms::core::begin_impl,
    typedef::Shared,
};

pub fn register(eval: &Evaluator) {
    eval.register_special_form("let", |args, env, eval| let_impl(args, env, eval, false));
    eval.register_special_form("let*", |args, env, eval| let_impl(args, env, eval, true));
}

fn let_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator, star: bool) -> EvalResult {
    if args.len() < 2 {
        return Err(SyntaxError::not_enough_args("let", 2, args.len()));
    }

    let Exp::List(binding) = &args[0] else {
        return Err(SyntaxError::invalid_args_type_nth("let", "list", 1));
    };

    let child_env = Env::new_child(Shared::clone(env));

    for pair in binding {
        match pair {
            Exp::List(list) if list.len() == 2 => {
                let Exp::Symbol(k) = &list[0] else {
                    return Err(SyntaxError::reason("expected symbol in binding"));
                };
                let v = if star {
                    eval.eval(&list[1], &child_env)
                } else {
                    eval.eval(&list[1], env)
                }?;
                let _ = &child_env.define(k, v.try_unwrap()?);
            }
            _ => return Err(SyntaxError::reason("invalid biding pair")),
        }
    }

    let body = &args[1..];
    begin_impl(body, &child_env, eval)
}
