use std::time::Duration;

use crate::{
    err,
    error::{Error, Result},
    evaluator::Evaluator,
    expression::Exp,
    future::FutureExp,
    typedef::SharedEnv,
};

/// 'call', 'range', 'sleep'
pub fn register(env: &SharedEnv) {
    env.define("call", Exp::Function(call_impl));
    env.define("range", Exp::Function(range_impl));

    #[cfg(feature = "async")]
    {
        env.define("sleep", Exp::Function(sleep_impl));
    }
}

fn call_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> Result<Exp> {
    if args.len() < 2 {
        err!("call expects (native 'method-name' ...args)")
    }
    let Exp::Native(native) = &args[0] else {
        err!("first argument must be a native object")
    };
    let Exp::String(method_name) = &args[1] else {
        err!("method name must be string")
    };
    native.call_method(&method_name, &args[2..])
}

fn range_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> Result<Exp> {
    if args.len() != 2 {
        Err(Error::from("expected exactly two numbers"))
    } else {
        let Exp::Number(first) = args[0] else {
            err!("range expected number")
        };
        let Exp::Number(second) = args[1] else {
            err!("range expected number")
        };
        let start = first as i64;
        let end = second as i64;
        Ok(Exp::List(
            (start..end)
                .into_iter()
                .map(|i| Exp::Number(i as f64))
                .collect(),
        ))
    }
}

#[cfg(feature = "async")]
fn sleep_impl(_: &[Exp], _: &SharedEnv, _: &Evaluator) -> Result<Exp> {
    use crate::GLOBAL_RUNTIME;
    // let handle = if tokio::runtime::Handle::try_current().is_ok() {
    //     tokio::runtime::Handle::current().spawn(async move {
    //         tokio::time::sleep(Duration::from_secs(10)).await;
    //         Ok(Exp::Bool(true))
    //     })
    // } else {
    //     GLOBAL_RUNTIME.spawn(async move {
    //         tokio::time::sleep(Duration::from_secs(10)).await;
    //         Ok(Exp::Bool(true))
    //     })
    // };

    let handle = GLOBAL_RUNTIME.spawn(async move {
        tokio::time::sleep(Duration::from_secs(10)).await;
        Ok(Exp::Bool(true))
    });
    Ok(Exp::Future(FutureExp::new(handle)))
}
