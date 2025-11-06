use crate::{
    environment::SharedEnv, error::SyntaxError, evaluator::Evaluator, exp::Exp, flow::EvalResult,
};

pub fn register(env: &SharedEnv) {
    env.define("expand-macro", Exp::Function(expand_macro_impl));
    env.define("dump-env", Exp::Function(dump_env_impl));
    env.define("trace-eval", Exp::Function(trace_eval_impl));
    env.define("native-keys", Exp::Function(native_keys_eval_impl));
}

fn expand_macro_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size(
            "expand-macro",
            1,
            args.len(),
        ));
    }
    let exp = args[0].clone();

    let Exp::List(list) = args[0].clone() else {
        return Ok(exp.clone().value_flow());
    };
    if list.is_empty() {
        return Ok(exp.clone().value_flow());
    }
    let Exp::Symbol(ref k) = list[0] else {
        return Ok(exp.clone().value_flow());
    };
    let Some(Exp::Macro(lambda)) = env.lookup(k) else {
        return Ok(exp.clone().value_flow());
    };
    let expanded = eval.expand_macro(lambda, &list[1..], env)?;
    Ok(expanded)
}

fn dump_env_impl(args: &[Exp], env: &SharedEnv, _: &Evaluator) -> EvalResult {
    env.dump(args.first().is_some_and(|e| e.is_truthy()));
    Ok(Exp::Bool(true).value_flow())
}

fn trace_eval_impl(args: &[Exp], _: &SharedEnv, eval: &Evaluator) -> EvalResult {
    eval.set_trace(args.first().is_none_or(|e| e.is_truthy()));
    Ok(Exp::Bool(true).value_flow())
}

fn native_keys_eval_impl(_: &[Exp], _: &SharedEnv, eval: &Evaluator) -> EvalResult {
    println!("native object keys:");
    for k in eval.native_object_creator_keys() {
        println!("   {}", k);
    }
    Ok(Exp::Bool(true).value_flow())
}
