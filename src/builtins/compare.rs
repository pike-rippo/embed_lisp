use crate::{
    builtins::math::parse_list_of_floats,
    err,
    error::Error,
    evaluator::Evaluator,
    expression::Exp,
    flow::{EvalFlow, EvalResult},
    typedef::SharedEnv,
};

macro_rules! ensure_tonicity {
    ($check_fn:expr) => {{
        |args: &[Exp], _: &SharedEnv, _: &Evaluator| -> EvalResult {
            let floats = parse_list_of_floats(args)?;
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
//ok!(f(first, rest))
/// '=', '>', '>=', '<', '<=', 'null?', 'eq?'
pub fn register(env: &SharedEnv) {
    env.define("=", Exp::Function(ensure_tonicity!(|a, b| a == b)));
    env.define(">", Exp::Function(ensure_tonicity!(|a, b| a > b)));
    env.define(">=", Exp::Function(ensure_tonicity!(|a, b| a >= b)));
    env.define("<", Exp::Function(ensure_tonicity!(|a, b| a < b)));
    env.define("<=", Exp::Function(ensure_tonicity!(|a, b| a <= b)));
    env.define("null?", Exp::Function(null_impl));
    env.define("eq?", Exp::Function(eq_impl));
}

fn null_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        err!("null? takes exactly one argument")
    }
    let Exp::List(list) = &args[0] else {
        err!("null? expects a list")
    };
    // ok!(list.is_empty())
    Ok(EvalFlow::Value(Exp::Bool(list.is_empty())))
}

fn eq_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        err!("eq? takes exactly two arguments")
    }
    // ok!(args[0] == args[1])
    Ok(EvalFlow::Value(Exp::Bool(args[0] == args[1])))
}
