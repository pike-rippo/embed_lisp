use std::time::Duration;

use crate::{
    Error,
    environment::SharedEnv,
    error::SyntaxError,
    evaluator::Evaluator,
    exp::Exp,
    flow::{EvalFlow, EvalResult},
    parser::Parser,
};

/// 'call', 'range', 'type-of', 'parse', 'eval'
pub fn register(env: &SharedEnv) {
    env.define("create-native", Exp::Primitive(create_native_impl));
    env.define("call", Exp::Primitive(call_impl));
    env.define("range", Exp::Primitive(range_impl));
    env.define("type-of", Exp::Primitive(type_of_impl));
    env.define("parse", Exp::Primitive(parse_impl));
    env.define("eval", Exp::Primitive(eval_impl));
    env.define("break", Exp::Primitive(break_impl));
    env.define("continue", Exp::Primitive(continue_impl));
    env.define("return", Exp::Primitive(return_impl));
    env.define("raise", Exp::Primitive(raise_impl));

    #[cfg(feature = "async")]
    {
        env.define("sleep", Exp::Primitive(sleep_impl));
    }
}

fn create_native_impl(args: &[Exp], _: &SharedEnv, eval: &Evaluator) -> EvalResult {
    let (name, args) = args
        .split_first()
        // .ok_or("create-native expected at least one argument")?;
        .ok_or(SyntaxError::not_enough_args("create-native", 2, args.len()))?;

    let Exp::String(name) = name else {
        // return Err("create-native expected a string as the first argument".into());
        return Err(SyntaxError::invalid_args_type_nth(
            "create-native",
            "string",
            1,
        ));
    };

    eval.create_native_object(name, args)
}

fn call_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() < 2 {
        return Err(SyntaxError::reason(
            "'call' expects (native 'method-name' ...args)",
        ));
    }
    let Exp::Native(native) = &args[0] else {
        return Err(SyntaxError::invalid_args_type_nth(
            "call",
            "native object",
            1,
        ));
    };
    let Exp::String(method_name) = &args[1] else {
        return Err(SyntaxError::invalid_args_type_nth("call", "string", 2));
    };
    native.call_method(method_name, &args[2..])
}

fn range_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        return Err(SyntaxError::invalid_args_size("range", 2, args.len()));
    } else {
        let Exp::Number(first) = args[0] else {
            return Err(SyntaxError::invalid_args_type("range", "number"));
        };
        let Exp::Number(second) = args[1] else {
            return Err(SyntaxError::invalid_args_type("range", "number"));
        };
        let start = first as i64;
        let end = second as i64;
        Ok(Exp::List((start..end).map(|i| Exp::Number(i as f64)).collect()).value_flow())
    }
}

fn type_of_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("type-of", 1, args.len()));
    }
    Ok(Exp::String(args[0].type_of()).value_flow())
}

fn parse_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("parse", 1, args.len()));
    }
    let Exp::String(input) = &args[0] else {
        return Err(SyntaxError::invalid_args_type("parse", "string"));
    };
    let exps = Parser::new().parse(input)?;
    Ok(Exp::List(exps).value_flow())
}

fn eval_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("eval", 1, args.len()));
    }

    // eval.eval(&args[0], env)
    match &args[0] {
        Exp::List(exps) => {
            let mut result = Exp::Nil.value_flow();
            for exp in exps {
                result = eval.eval(exp, env)?;
            }
            Ok(result)
        }
        other => eval.eval(other, env),
    }
}

fn break_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if !args.is_empty() {
        // err!("'break' expected no arguments");
        return Err(SyntaxError::invalid_args_size("break", 0, args.len()));
    }
    Ok(EvalFlow::Break)
}
fn continue_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if !args.is_empty() {
        return Err(SyntaxError::invalid_args_size("continue", 0, args.len()));
    }
    Ok(EvalFlow::Continue)
}
fn return_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() > 1 {
        return Err(SyntaxError::reason(
            "'return' expected at most one argument",
        ));
    }
    Ok(EvalFlow::Return(args.get(0).cloned().unwrap_or(Exp::Nil)))
}

fn raise_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() > 1 {
        return Err(SyntaxError::invalid_args_size("raise", 1, args.len()));
    }

    let Exp::String(msg) = &args[0] else {
        return Err(SyntaxError::invalid_args_type("raise", "string"));
    };

    Err(Error::Reason(msg.clone()))
}

#[cfg(feature = "async")]
fn sleep_impl(_: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    #[cfg(feature = "async")]
    use crate::GLOBAL_RUNTIME;
    use crate::exp::TaskExp;

    let handle = GLOBAL_RUNTIME.spawn(async move {
        tokio::time::sleep(Duration::from_secs(10)).await;
        Ok(Exp::Bool(true).value_flow())
    });
    Ok(Exp::Task(TaskExp::new(handle)).value_flow())
}
