use crate::{
    environment::EnvRc,
    err,
    error::{Error, Result},
    evaluator::Evaluator,
    expression::Exp,
};

/// 'call'
pub fn register(env: &EnvRc) {
    env.define("call", Exp::Function(call_impl));
    env.define("range", Exp::Function(range_impl));
}

fn call_impl(args: &[Exp], _: &EnvRc, _: &Evaluator) -> Result<Exp> {
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

fn range_impl(args: &[Exp], _: &EnvRc, _: &Evaluator) -> Result<Exp> {
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
