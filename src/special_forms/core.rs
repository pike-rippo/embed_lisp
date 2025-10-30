use crate::{
    environment::Env,
    err,
    evaluator::Evaluator,
    expression::Exp,
    flow::{EvalFlow, EvalResult},
    lambda::LambdaExp,
    typedef::{Shared, SharedEnv},
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

    #[cfg(feature = "async")]
    {
        eval.register_special_form("async", async_impl);
        eval.register_special_form("spawn", spawn_impl);
        eval.register_special_form("await", await_impl);
    }
}

fn define_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        err!("define cam only two forms")
    }

    let Some(Exp::Symbol(k)) = args.first() else {
        err!("expected first form to be a symbol")
    };

    let v = eval.eval(args.get(1).unwrap(), env)?;
    Ok(env.define(k, v.try_unwrap()?).value_flow())
}

fn assign_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        err!("assign cam only two forms")
    }

    let Some(Exp::Symbol(k)) = args.first() else {
        err!("expected first form to be a symbol")
    };

    let v = eval.eval(args.get(1).unwrap(), env)?;
    Ok(env.assign(k, v.try_unwrap()?).value_flow())
}

fn lambda_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() < 2 {
        err!("lambda definition can only have two forms")
    }

    let Some(params_exp) = args.first() else {
        err!("expected args form")
    };
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
        err!("quote can only have one form")
    } else {
        Ok(args[0].clone().value_flow())
    }
}

fn quasiquote_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        err!("quasiquote can only have one form")
    }

    let (exp, _splicing) = eval.eval_quasiquote(&args[0], env)?;
    Ok(exp.value_flow())
}

fn for_each_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    if args.len() < 2 {
        err!("for expects at least 2 arguments")
    }

    let binding_pair = match &args[0] {
        Exp::List(list) if list.len() == 2 => list,
        _ => err!("for expects first argument must be binding list"),
    };

    let Exp::Symbol(k) = &binding_pair[0] else {
        err!("binding list must start with a symbol")
    };

    let EvalFlow::Value(Exp::List(values)) = eval.eval(&binding_pair[1], env)? else {
        err!("binding list must end with a list")
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

#[cfg(feature = "async")]
fn async_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    use crate::future::FutureExp;

    if args.len() != 1 {
        err!("async can only have one form")
    }

    Ok(Exp::Future(FutureExp::new(&args[0], env, eval)).value_flow())
}

#[cfg(feature = "async")]
fn spawn_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    use crate::task::TaskExp;

    if args.len() != 1 {
        err!("spawn can only have one form")
    }

    let handle = match &args[0] {
        Exp::Future(future) => future.spawn(),
        other => match eval.eval(other, env)? {
            EvalFlow::Value(Exp::Future(future)) => future.spawn(),
            _ => err!("spawn can only have one future"),
        },
    };

    Ok(Exp::Task(TaskExp::new(handle)).value_flow())
}

#[cfg(feature = "async")]
fn await_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        err!("await can only have one form")
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
