use crate::{
    environment::Env,
    environment::SharedEnv,
    error::SyntaxError,
    evaluator::Evaluator,
    exp::{Exp, LambdaExp},
    flow::{EvalFlow, EvalResult},
    typedef::Shared,
};
/// 'lambda', 'begin', 'quote', 'for'
pub fn register(eval: &Evaluator) {
    eval.register_special_form("lambda", lambda_impl);
    eval.register_special_form("recur-lambda", recur_lambda_impl);
    eval.register_special_form("begin", begin_impl);
    eval.register_special_form("quote", quote_impl);
    eval.register_special_form("quasiquote", quasiquote_impl);
    eval.register_special_form("for-each", for_each_impl);
    eval.register_special_form("loop", loop_impl);
    eval.register_special_form("gensym", gensym_impl);
    eval.register_special_form("scope", scope_impl);
    eval.register_special_form("stringify", stringify_impl);
    eval.register_special_form("recur", recur_impl);
    eval.register_special_form("try", try_impl);

    #[cfg(feature = "async")]
    {
        eval.register_special_form("async", async_impl);
        eval.register_special_form("spawn", spawn_impl);
        eval.register_special_form("await", await_impl);
    }
}

fn lambda_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() < 2 {
        return Err(SyntaxError::not_enough_args("lambda", 2, args.len()));
    }

    let params_exp = &args[0];
    let body = &args[1..];

    Ok(Exp::Lambda(LambdaExp::new(
        Shared::new(params_exp.clone()),
        Vec::from(body),
    ))
    .value_flow())
}

fn recur_lambda_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() < 3 {
        return Err(SyntaxError::not_enough_args("recur-lambda", 3, args.len()));
    }

    let init = &args[0];
    let params_exp = &args[1];
    let body = &args[2..];

    Ok(Exp::RecurLambda(
        Shared::new(init.clone()),
        LambdaExp::new(Shared::new(params_exp.clone()), Vec::from(body)),
    )
    .value_flow())
}

pub fn begin_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    let mut last = Exp::Nil;

    for form in args {
        match eval.eval(form, env)? {
            EvalFlow::Value(v) => last = v,
            otherwise => return Ok(otherwise),
        }
    }

    Ok(last.value_flow())
}

fn quote_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::not_enough_args("quote", 1, args.len()));
    } else {
        Ok(args[0].clone().value_flow())
    }
}

fn quasiquote_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::not_enough_args("quasiquote", 1, args.len()));
    }

    let (exp, _splicing) = eval.eval_quasiquote(&args[0], env)?;
    Ok(exp.value_flow())
}

fn for_each_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    if args.len() < 2 {
        return Err(SyntaxError::not_enough_args("for", 2, args.len()));
    }

    let binding_pair = match &args[0] {
        Exp::List(list) if list.len() == 2 => list,
        _ => {
            return Err(SyntaxError::reason(
                "'for' expects first argument must be binding list",
            ));
        }
    };

    let Exp::Symbol(k) = &binding_pair[0] else {
        return Err(SyntaxError::reason("binding list must start with a symbol"));
    };

    let EvalFlow::Value(Exp::List(values)) = eval.eval(&binding_pair[1], env)? else {
        return Err(SyntaxError::reason("binding list must end with a list"));
    };
    let mut result = Exp::Nil;
    let new_env = Env::new_child(Shared::clone(env));
    for value in values {
        new_env.define(k, value.clone());
        match begin_impl(&args[1..], &new_env, eval)? {
            EvalFlow::Value(exp) => result = exp,
            EvalFlow::Continue => continue,
            EvalFlow::Break => return Ok(Exp::Nil.value_flow()),
            otherwise => return Ok(otherwise),
        }
    }
    Ok(result.value_flow())
}

fn loop_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    loop {
        match begin_impl(args, env, eval)? {
            EvalFlow::Value(_) => {}
            EvalFlow::Continue => continue,
            EvalFlow::Break => return Ok(Exp::Nil.value_flow()),
            otherwise => return Ok(otherwise),
        }
    }
}

fn gensym_impl(_args: &[Exp], _: &SharedEnv, eval: &Evaluator) -> EvalResult {
    Ok(Exp::Symbol(format!("__GEN_SYM__{}", eval.get_gensym_id())).value_flow())
}

fn scope_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    let child = Env::new_child(env.clone());
    begin_impl(args, &child, eval)
}

fn stringify_impl(args: &[Exp], _env: &SharedEnv, _eval: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("stringify", 1, args.len()));
    }

    Ok(Exp::String(format!("{}", &args[0])).value_flow())
}

fn recur_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    let values = eval.eval_form(args, env)?;
    Ok(EvalFlow::TailCall(values))
}

fn try_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        return Err(SyntaxError::not_enough_args("try", 2, args.len()));
    }

    let try_block = &args[0];
    let catch_form = &args[1];

    let Exp::List(catch_parts) = catch_form else {
        return Err(SyntaxError::invalid_args_type("catch form", "list"));
    };

    if catch_parts.len() < 2 {
        return Err(SyntaxError::not_enough_args("catch", 2, catch_parts.len()));
    }

    let Exp::Symbol(_) = &catch_parts[0] else {
        return Err(SyntaxError::invalid_args_type("catch", "symbol"));
    };

    let Exp::Symbol(var_name) = &catch_parts[1] else {
        return Err(SyntaxError::invalid_args_type_nth("catch", "symbol", 2));
    };

    let catch_body = &catch_parts[2..];
    match eval.eval(try_block, env) {
        Ok(v) => Ok(v),
        Err(e) => {
            let child = Env::new_child(env.clone());
            child.define(var_name, Exp::String(e.to_string()));
            begin_impl(catch_body, &child, eval)
        }
    }
}

#[cfg(feature = "async")]
fn async_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    use crate::exp::FutureExp;

    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("async", 1, args.len()));
    }

    Ok(Exp::Future(FutureExp::new(&args[0], env, eval)).value_flow())
}

#[cfg(feature = "async")]
fn spawn_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    use crate::exp::TaskExp;

    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("spawn", 1, args.len()));
    }

    let handle = match &args[0] {
        Exp::Future(future) => future.spawn(),
        other => match eval.eval(other, env)? {
            EvalFlow::Value(Exp::Future(future)) => future.spawn(),
            _ => return Err(SyntaxError::invalid_args_type("spawn", "future")),
        },
    };

    Ok(Exp::Task(TaskExp::new(handle)).value_flow())
}

#[cfg(feature = "async")]
fn await_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("await", 1, args.len()));
    }

    match &args[0] {
        Exp::Symbol(k) => {
            use crate::error::Error;

            let value = env
                .lookup(k)
                .ok_or(Error::Reason(format!("unexpected symbol '{}'", k)))?;

            if let Exp::Task(f) = value {
                let result = if tokio::runtime::Handle::try_current().is_ok() {
                    tokio::runtime::Handle::current().block_on(f.async_await())
                } else {
                    f.sync_await()
                }?;
                env.assign(k, result.try_unwrap()?);
                Ok(result)
            } else {
                Ok(value.value_flow())
            }
        }
        other => match eval.eval(other, env)? {
            EvalFlow::Value(Exp::Task(task)) => {
                if tokio::runtime::Handle::try_current().is_ok() {
                    tokio::task::block_in_place(|| task.sync_await())
                } else {
                    task.sync_await()
                }
            }
            exp => Ok(exp),
        },
    }
}
