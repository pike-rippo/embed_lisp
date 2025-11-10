use crate::{
    environment::{Env, SharedEnv},
    error::SyntaxError,
    evaluator::Evaluator,
    exp::Exp,
    flow::EvalResult,
    special_forms::core::begin_impl,
    typedef::Shared,
};

pub fn register(eval: &Evaluator) {
    eval.register_special_form("define", define_impl);
    eval.register_special_form("define-at", define_at_impl);
    eval.register_special_form("define-parent", define_parent_impl);
    eval.register_special_form("assign", assign_impl);
    eval.register_special_form("drop", drop_impl);
    eval.register_special_form("let", |args, env, eval| let_impl(args, env, eval, false));
    eval.register_special_form("let*", |args, env, eval| let_impl(args, env, eval, true));
}

fn define_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        return Err(SyntaxError::invalid_args_size("define", 2, args.len()));
    }

    let Some(Exp::Symbol(k)) = args.first() else {
        return Err(SyntaxError::invalid_args_type_nth("define", "symbol", 1));
    };

    let v = eval.eval(&args[1], env)?;
    Ok(env.define(k, v.try_unwrap()?).value_flow())
}

fn define_at_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    if args.len() != 3 {
        return Err(SyntaxError::invalid_args_size("define-at", 2, args.len()));
    }

    let level = match &args[0] {
        Exp::Symbol(s) if s.eq_ignore_ascii_case("builtin") => 0,
        Exp::Symbol(s) if s.eq_ignore_ascii_case("system") => 1,
        Exp::Symbol(s) if s.eq_ignore_ascii_case("global") => 2,
        Exp::Number(n) if *n >= 0.0 => *n as usize + 2,
        _ => {
            return Err(SyntaxError::invalid_args_type_nth(
                "define-at",
                "positive number",
                1,
            ));
        }
    };

    let Exp::Symbol(k) = &args[1] else {
        return Err(SyntaxError::invalid_args_type_nth("define-at", "symbol", 2));
    };

    let v = eval.eval(&args[2], env)?;
    Ok(env.define_at(level, k, v.try_unwrap()?).value_flow())
}

fn define_parent_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    if args.len() != 3 {
        return Err(SyntaxError::invalid_args_size("define-at", 2, args.len()));
    }

    let Exp::Number(level) = &args[0] else {
        return Err(SyntaxError::invalid_args_type_nth(
            "define-at",
            "positive number",
            1,
        ));
    };

    let level = if *level >= 0.0 {
        *level as usize
    } else {
        return Err(SyntaxError::invalid_args_type_nth(
            "define-at",
            "positive number",
            1,
        ));
    };

    let Exp::Symbol(k) = &args[1] else {
        return Err(SyntaxError::invalid_args_type_nth("define-at", "symbol", 2));
    };

    let v = eval.eval(&args[2], env)?;
    Ok(env.define_at_parent(level, k, v.try_unwrap()?).value_flow())
}

fn assign_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        return Err(SyntaxError::invalid_args_size("assign", 2, args.len()));
    }

    let Some(Exp::Symbol(k)) = args.first() else {
        return Err(SyntaxError::invalid_args_type_nth("assign", "symbol", 1));
    };

    let v = eval.eval(args.get(1).unwrap(), env)?;
    Ok(env.assign(k, v.try_unwrap()?).value_flow())
}

fn drop_impl(args: &[Exp], env: &SharedEnv, _eval: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("drop", 1, args.len()));
    }

    let Exp::Symbol(k) = args.first().unwrap() else {
        return Err(SyntaxError::invalid_args_type("drop", "symbol"));
    };

    Ok(env.drop_symbol(k).value_flow())
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
