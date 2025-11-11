use crate::{
    builtins::math::parse_list_of_floats,
    environment::SharedEnv,
    error::{Error, SyntaxError},
    evaluator::Evaluator,
    exp::Exp,
    flow::{EvalFlow, EvalResult},
};

macro_rules! ensure_tonicity {
    ($name:expr, $check_fn:expr) => {{
        |args: &[Exp], _: &SharedEnv, _: &Evaluator| -> EvalResult {
            let floats = parse_list_of_floats($name, args)?;
            let first = floats
                .first()
                .ok_or(Error::Reason("expected at least one number".to_string()))?;
            let rest = &floats[1..];
            fn f(prev: &f64, xs: &[f64]) -> bool {
                match xs.first() {
                    Some(x) => $check_fn(prev, x) && f(x, &xs[1..]),
                    None => true,
                }
            }
            Ok(EvalFlow::Value(Exp::Bool(f(first, rest))))
        }
    }};
}

pub fn register(env: &SharedEnv) {
    env.define("=", Exp::Primitive(ensure_tonicity!("=", |a, b| a == b)));
    env.define(">", Exp::Primitive(ensure_tonicity!(">", |a, b| a > b)));
    env.define(">=", Exp::Primitive(ensure_tonicity!(">=", |a, b| a >= b)));
    env.define("<", Exp::Primitive(ensure_tonicity!("<", |a, b| a < b)));
    env.define("<=", Exp::Primitive(ensure_tonicity!("<=", |a, b| a <= b)));
    env.define("null?", Exp::Primitive(null_impl));
    env.define("eq?", Exp::Primitive(eq_impl));
}

fn null_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("null?", 0, args.len()));
    }
    let Exp::List(list) = &args[0] else {
        return Err(SyntaxError::invalid_args_type_nth("null?", "list", 1));
    };
    Ok(EvalFlow::Value(Exp::Bool(list.is_empty())))
}

fn eq_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        return Err(SyntaxError::invalid_args_size("eq?", 2, args.len()));
    }
    Ok(EvalFlow::Value(Exp::Bool(args[0] == args[1])))
}
