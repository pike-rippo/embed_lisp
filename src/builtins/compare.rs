use crate::{
    builtins::math::parse_list_of_floats,
    environment::EnvRc,
    err,
    error::{Error, Result},
    evaluator::Evaluator,
    expression::Exp,
    ok,
};

macro_rules! ensure_tonicity {
    ($check_fn:expr) => {{
        |args: &[Exp], _: &EnvRc, _: &Evaluator| -> Result<Exp> {
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
            ok!(f(first, rest))
        }
    }};
}

/// '=', '>', '>=', '<', '<=', 'null?', 'eq?'
pub fn register(env: &EnvRc) {
    env.define("=", Exp::Function(ensure_tonicity!(|a, b| a == b)));
    env.define(">", Exp::Function(ensure_tonicity!(|a, b| a > b)));
    env.define(">=", Exp::Function(ensure_tonicity!(|a, b| a >= b)));
    env.define("<", Exp::Function(ensure_tonicity!(|a, b| a < b)));
    env.define("<=", Exp::Function(ensure_tonicity!(|a, b| a <= b)));
    env.define("null?", Exp::Function(null_impl));
    env.define("eq?", Exp::Function(eq_impl));
}

fn null_impl(args: &[Exp], _: &EnvRc, _: &Evaluator) -> Result<Exp> {
    if args.len() != 1 {
        err!("null? takes exactly one argument")
    }
    let Exp::List(list) = &args[0] else {
        err!("null? expects a list")
    };
    ok!(list.len() == 0)
}

fn eq_impl(args: &[Exp], _: &EnvRc, _: &Evaluator) -> Result<Exp> {
    if args.len() != 2 {
        err!("eq? takes exactly two arguments")
    }
    ok!(args[0] == args[1])
}
