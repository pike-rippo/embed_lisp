use std::rc::Rc;

use crate::{
    environment::Env,
    err,
    error::{Error, Result},
    evaluator::Evaluator,
    expression::Exp,
    lambda::LambdaExp,
    typedef::{Shared, SharedEnv},
};
#[cfg(feature = "async")]
/// 'define', 'assign', 'lambda', 'begin', 'quote', 'for'
pub fn register(eval: &Evaluator) {
    eval.register_special_form("define", define_impl);
    eval.register_special_form("assign", assign_impl);
    eval.register_special_form("lambda", lambda_impl);
    eval.register_special_form("begin", begin_impl);
    eval.register_special_form("quote", quote_impl);
    eval.register_special_form("quasiquote", quasiquote_impl);
    eval.register_special_form("for", for_impl);

    #[cfg(feature = "async")]
    {
        eval.register_special_form("async", async_impl);
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

    //listならevalしてもいいかも
    let Exp::Symbol(k) = &binding_pair[0] else {
        err!("binding list must start with a symbol")
    };

    let Exp::List(values) = eval.eval(&binding_pair[1], env)? else {
        err!("binding list must end with a list")
    };
    let mut result = Exp::Nil;
    for value in values {
        let new_env = Env::new_child(Shared::clone(&env));
        new_env.define(&k, value.clone());
        result = eval.eval(&args[1], &new_env)?;
    }
    Ok(result)
}

#[cfg(feature = "async")]
fn async_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> Result<Exp> {
    use crate::{GLOBAL_RUNTIME, future::FutureExp};

    if args.len() != 1 {
        err!("async can only have one form")
    }

    let arg = args[0].clone();
    let copy_env = env.deep_copy();
    let copy_eval = eval.deep_copy();
    let handle = GLOBAL_RUNTIME.spawn(async move { copy_eval.eval(&arg, &copy_env) });

    Ok(Exp::Future(FutureExp::new(handle)))
}

#[cfg(feature = "async")]
fn await_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> Result<Exp> {
    if args.len() != 1 {
        err!("await can only have one form")
    }

    if let Exp::Symbol(k) = &args[0] {
        let value = env
            .lookup(&k)
            .ok_or(Error::Reason(format!("unexpected symbol '{}'", k)))?;

        if let Exp::Future(f) = value {
            // let result = f.sync_await()?;
            let result = if tokio::runtime::Handle::try_current().is_ok() {
                tokio::runtime::Handle::current().block_on(f.async_await())
            } else {
                f.sync_await()
            }?;
            env.assign(k, result.clone());
            return Ok(result);
        } else {
            // err!(format!("{} is not a FutureExp", k));
            return Ok(value);
        }
    }

    let val = eval.eval(&args[0], env)?;
    if let Exp::Future(f) = val {
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::block_in_place(|| f.sync_await())
        } else {
            f.sync_await()
        }
    } else {
        // err!("await: argument is not a FutureExp")
        return Ok(val);
    }
}
