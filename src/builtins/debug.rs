use crate::{err, error::Result, evaluator::Evaluator, expression::Exp, ok, typedef::SharedEnv};

pub fn register(env: &SharedEnv) {
    env.define("expand-macro", Exp::Function(expand_macro_impl));
    env.define("dump-env", Exp::Function(dump_env_impl));
    env.define("trace-eval", Exp::Function(trace_eval_impl));
}

fn expand_macro_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> Result<Exp> {
    if args.len() != 1 {
        err!("expand-macro expects exactly one argument")
    }
    let exp = args[0].clone();

    let Exp::List(list) = args[0].clone() else {
        return Ok(exp.clone());
    };
    if list.is_empty() {
        return Ok(exp.clone());
    }
    let Exp::Symbol(ref k) = list[0] else {
        return Ok(exp.clone());
    };
    let Some(Exp::Macro(lambda)) = env.lookup(k) else {
        return Ok(exp.clone());
    };
    let expanded = eval.expand_macro(lambda, &list[1..], env)?;
    Ok(expanded)
}

fn dump_env_impl(args: &[Exp], env: &SharedEnv, _: &Evaluator) -> Result<Exp> {
    env.dump(args.get(0).map_or(false, |e| e.is_truthy()));
    ok!(true)
}

fn trace_eval_impl(args: &[Exp], _: &SharedEnv, eval: &Evaluator) -> Result<Exp> {
    // eval.set_trace(true);
    eval.set_trace(args.get(0).map_or(false, |e| e.is_truthy()));
    ok!(true)
}
