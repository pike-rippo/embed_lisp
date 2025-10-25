use std::time::Duration;

use crate::{
    err,
    error::{Error, Result},
    evaluator::Evaluator,
    expression::Exp,
    parser::Parser,
    typedef::SharedEnv,
};

/// 'call', 'range', 'type-of', 'parse', 'eval'
pub fn register(env: &SharedEnv) {
    env.define("call", Exp::Function(call_impl));
    env.define("range", Exp::Function(range_impl));
    env.define("type-of", Exp::Function(type_of_impl));
    env.define("parse", Exp::Function(parse_impl));
    env.define("eval", Exp::Function(eval_impl));

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
    native.call_method(method_name, &args[2..])
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
            (start..end).map(|i| Exp::Number(i as f64)).collect(),
        ))
    }
}

fn type_of_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> Result<Exp> {
    if args.len() != 1 {
        err!("type-of expected one argument");
    }
    Ok(Exp::String(args[0].type_of()))
}

fn parse_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> Result<Exp> {
    if args.len() != 1 {
        err!("parse expected one argument");
    }
    let Exp::String(input) = &args[0] else {
        err!("parse expected string")
    };
    let Ok(exps) = Parser::new().parse(input) else {
        err!("parse failed")
    };
    Ok(Exp::List(exps))
}

fn eval_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> Result<Exp> {
    if args.len() != 1 {
        err!("eval expected one argument");
    }

    // eval.eval(&args[0], env)
    match &args[0] {
        Exp::List(exps) => {
            let mut result = Exp::Nil;
            for exp in exps {
                result = eval.eval(exp, env)?;
            }
            Ok(result)
        }
        other => eval.eval(other, env),
    }
}

#[cfg(feature = "async")]
fn sleep_impl(_: &[Exp], _: &SharedEnv, _: &Evaluator) -> Result<Exp> {
    #[cfg(feature = "async")]
    use crate::GLOBAL_RUNTIME;
    use crate::task::TaskExp;

    let handle = GLOBAL_RUNTIME.spawn(async move {
        tokio::time::sleep(Duration::from_secs(10)).await;
        Ok(Exp::Bool(true))
    });
    Ok(Exp::Task(TaskExp::new(handle)))
}
