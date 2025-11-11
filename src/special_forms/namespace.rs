use crate::{
    Evaluator, Exp, environment::SharedEnv, error::SyntaxError, exp::NameSpace, flow::EvalResult,
    typedef::Shared,
};

pub fn register(eval: &Evaluator) {
    eval.register_special_form("namespace", namespace_impl);
    eval.register_special_form("ns-define", ns_define_impl);
}

fn namespace_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    let ns = Exp::Namespace(Shared::new(NameSpace::default()));
    if args.is_empty() {
        return Ok(ns.value_flow());
    }

    for arg in args {
        if let Exp::List(list) = arg {
            let mut params = Vec::with_capacity(list.len() + 1);
            params.push(ns.clone());
            params.extend(list.clone());
            ns_define_impl(&params, env, eval)?;
        } else {
            return Err(SyntaxError::invalid_args_type("namespace", "list"));
        }
    }

    Ok(ns.value_flow())
}

fn ns_define_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    if args.len() != 3 {
        return Err(SyntaxError::invalid_args_size("ns-define", 3, args.len()));
    }

    let Exp::Namespace(ns) = eval.eval(&args[0], env)?.try_unwrap()? else {
        return Err(SyntaxError::invalid_args_type_nth(
            "ns-define",
            "namespace",
            1,
        ));
    };

    let Exp::Symbol(k) = &args[1] else {
        return Err(SyntaxError::invalid_args_type_nth("ns-define", "symbol", 2));
    };

    let v = eval.eval(&args[2], env)?;
    Ok(ns.define(k, v.try_unwrap()?).value_flow())
}
