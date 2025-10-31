use crate::{
    environment::Env,
    environment::SharedEnv,
    error::SyntaxError,
    evaluator::Evaluator,
    expression::Exp,
    flow::{EvalFlow, EvalResult},
    lambda::LambdaExp,
    typedef::Shared,
};
/// 'define', 'assign', 'lambda', 'begin', 'quote', 'for'
pub fn register(eval: &Evaluator) {
    eval.register_special_form("define", define_impl);
    eval.register_special_form("assign", assign_impl);
    eval.register_special_form("lambda", lambda_impl);
    eval.register_special_form("begin", begin_impl);
    eval.register_special_form("quote", quote_impl);
    eval.register_special_form("quasiquote", quasiquote_impl);
    eval.register_special_form("for-each", for_each_impl);
    eval.register_special_form("loop", loop_impl);
    eval.register_special_form("gensym", gensym_impl);
    eval.register_special_form("scope", scope_impl);

    #[cfg(feature = "async")]
    {
        eval.register_special_form("async", async_impl);
        eval.register_special_form("spawn", spawn_impl);
        eval.register_special_form("await", await_impl);
    }
}

fn define_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        return Err(SyntaxError::invalid_args_size("define", 2, args.len()));
    }

    let Some(Exp::Symbol(k)) = args.first() else {
        return Err(SyntaxError::invalid_args_type_nth("define", "symbol", 1));
    };

    let v = eval.eval(args.get(1).unwrap(), env)?;
    Ok(env.define(k, v.try_unwrap()?).value_flow())
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

pub fn begin_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    let mut last = Exp::Nil;

    for form in args {
        match eval.eval(form, env)? {
            EvalFlow::Value(v) => last = v,
            flow @ EvalFlow::Break => return Ok(flow),
            flow @ EvalFlow::Continue => return Ok(flow),
            flow @ EvalFlow::Return(_) => return Ok(flow),
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
            flow @ EvalFlow::Return(_) => return Ok(flow),
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
            flow @ EvalFlow::Return(_) => return Ok(flow),
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

#[cfg(feature = "async")]
fn async_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    use crate::future::FutureExp;

    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("async", 1, args.len()));
    }

    Ok(Exp::Future(FutureExp::new(&args[0], env, eval)).value_flow())
}

#[cfg(feature = "async")]
fn spawn_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    use crate::task::TaskExp;

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
