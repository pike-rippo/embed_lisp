use crate::{
    environment::SharedEnv,
    error::SyntaxError,
    evaluator::Evaluator,
    exp::{Exp, NameSpace},
    flow::EvalResult,
    typedef::Shared,
};

pub fn register(env: &SharedEnv) {
    env.define("create-namespace", Exp::Primitive(create_namespace_impl));
    env.define("ns-get", Exp::Primitive(ns_get_impl));
    env.define("use", Exp::Primitive(use_impl));
}

fn create_namespace_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if !args.is_empty() {
        return Err(SyntaxError::invalid_args_size(
            "create-namespace",
            0,
            args.len(),
        ));
    }

    Ok(Exp::Namespace(Shared::new(NameSpace::default())).value_flow())
}

fn ns_get_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        return Err(SyntaxError::invalid_args_size("ns-get", 2, args.len()));
    }

    let Exp::Namespace(ns) = &args[0] else {
        return Err(SyntaxError::invalid_args_type_nth("ns-get", "namespace", 1));
    };

    let Exp::String(k) = &args[1] else {
        return Err(SyntaxError::invalid_args_type_nth("ns-get", "string", 2));
    };

    Ok(ns.try_lookup(k)?.value_flow())
}

fn use_impl(args: &[Exp], env: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("use", 1, args.len()));
    }

    let Exp::Namespace(ns) = &args[0] else {
        return Err(SyntaxError::invalid_args_type_nth("use", "namespace", 1));
    };

    ns.expand(env);

    Ok(Exp::Bool(true).value_flow())
}
