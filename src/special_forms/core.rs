use crate::{
    environment::Env,
    err,
    error::Result,
    evaluator::Evaluator,
    expression::Exp,
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
    eval.register_special_form("for", for_impl);
    eval.register_special_form("gensym", gensym_impl);

    #[cfg(feature = "async")]
    {
        eval.register_special_form("async", async_impl);
        eval.register_special_form("spawn", spawn_impl);
        eval.register_special_form("await", await_impl);
    }
}

fn define_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> Result<Exp> {
    if args.len() != 2 {
        err!("define cam only two forms")
    }

    let Some(Exp::Symbol(k)) = args.first() else {
        err!("expected first form to be a symbol")
    };

    let v = eval.eval(args.get(1).unwrap(), env)?;
    Ok(env.define(k, v))
}

fn assign_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> Result<Exp> {
    if args.len() != 2 {
        err!("assign cam only two forms")
    }

    let Some(Exp::Symbol(k)) = args.first() else {
        err!("expected first form to be a symbol")
    };

    let v = eval.eval(args.get(1).unwrap(), env)?;
    Ok(env.assign(k, v))
}

fn lambda_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> Result<Exp> {
    if args.len() != 2 {
        err!("lambda definition can only have two forms")
    }

    let Some(params_exp) = args.first() else {
        err!("expected args form")
    };
    let Some(body_exp) = args.get(1) else {
        err!("expected body exp")
    };

    Ok(Exp::Lambda(LambdaExp::new(
        Shared::new(params_exp.clone()),
        Shared::new(body_exp.clone()),
    )))
}

pub fn begin_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> Result<Exp> {
    let mut last = Exp::Nil;

    for form in args {
        let val = eval.eval(form, env)?;
        // if val.is_control_flow() {
        //     ok!(val)
        // }
        last = val;
    }

    Ok(last)
}

fn quote_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> Result<Exp> {
    if args.len() != 1 {
        err!("quote can only have one form")
    } else {
        Ok(args[0].clone())
    }
}

fn quasiquote_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> Result<Exp> {
    if args.len() != 1 {
        err!("quasiquote can only have one form")
    }

    eval.eval_quasiquote(&args[0], env)
}

fn for_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> Result<Exp> {
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

    let Exp::List(values) = eval.eval(&binding_pair[1], env)? else {
        err!("binding list must end with a list")
    };
    let mut result = Exp::Nil;
    let new_env = Env::new_child(Shared::clone(&env));
    for value in values {
        new_env.define(&k, value.clone());
        result = eval.eval(&args[1], &new_env)?;
    }
    Ok(result)
}

fn gensym_impl(args: &[Exp], _: &SharedEnv, eval: &Evaluator) -> Result<Exp> {
    Ok(Exp::Symbol(format!("__GEN_SYM__{}", eval.get_gensym_id())))
}

#[cfg(feature = "async")]
fn async_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> Result<Exp> {
    use crate::future::FutureExp;

    if args.len() != 1 {
        err!("async can only have one form")
    }

    Ok(Exp::Future(FutureExp::new(&args[0], &env, &eval)))
}

#[cfg(feature = "async")]
fn spawn_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> Result<Exp> {
    use crate::task::TaskExp;

    if args.len() != 1 {
        err!("spawn can only have one form")
    }

    // let Exp::Future(future) = &args[0] else {
    //     err!("spawn can only have one future")
    // };

    // Ok(Exp::Task(TaskExp::new(future.spawn())))

    let handle = match &args[0] {
        Exp::Future(future) => future.spawn(),
        other => match eval.eval(other, env)? {
            Exp::Future(future) => future.spawn(),
            _ => err!("spawn can only have one future"),
        },
    };

    Ok(Exp::Task(TaskExp::new(handle)))
}

#[cfg(feature = "async")]
fn await_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> Result<Exp> {
    if args.len() != 1 {
        err!("await can only have one form")
    }

    match &args[0] {
        Exp::Symbol(k) => {
            use crate::error::Error;

            let value = env
                .lookup(&k)
                .ok_or(Error::Reason(format!("unexpected symbol '{}'", k)))?;

            if let Exp::Task(f) = value {
                let result = if tokio::runtime::Handle::try_current().is_ok() {
                    tokio::runtime::Handle::current().block_on(f.async_await())
                } else {
                    f.sync_await()
                }?;
                env.assign(k, result.clone());
                Ok(result)
            } else {
                Ok(value)
            }
        }
        other => match eval.eval(other, env)? {
            Exp::Task(task) => {
                if tokio::runtime::Handle::try_current().is_ok() {
                    tokio::task::block_in_place(|| task.sync_await())
                } else {
                    task.sync_await()
                }
            }
            exp => Ok(exp),
        },
    }

    // if let Exp::Symbol(k) = &args[0] {
    //     let value = env
    //         .lookup(&k)
    //         .ok_or(Error::Reason(format!("unexpected symbol '{}'", k)))?;

    //     if let Exp::Task(f) = value {
    //         let result = if tokio::runtime::Handle::try_current().is_ok() {
    //             tokio::runtime::Handle::current().block_on(f.async_await())
    //         } else {
    //             f.sync_await()
    //         }?;
    //         env.assign(k, result.clone());
    //         return Ok(result);
    //     } else {
    //         return Ok(value);
    //     }
    // }

    // let val = eval.eval(&args[0], env)?;
    // if let Exp::Task(f) = val {
    //     if tokio::runtime::Handle::try_current().is_ok() {
    //         tokio::task::block_in_place(|| f.sync_await())
    //     } else {
    //         f.sync_await()
    //     }
    // } else {
    //     return Ok(val);
    // }
}
