use crate::{
    err,
    evaluator::Evaluator,
    expression::Exp,
    flow::{EvalFlow, EvalResult},
    special_forms::core::begin_impl,
    typedef::SharedEnv,
};

/// 'if', 'cond', 'and', 'or'
pub fn register(eval: &Evaluator) {
    eval.register_special_form("if", if_impl);
    eval.register_special_form("cond", cond_impl);
    eval.register_special_form("and", and_impl);
    eval.register_special_form("or", or_impl);
}

fn if_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    let Some(test_form) = args.first() else {
        err!("expected test form")
    };
    let test_eval = eval.eval(test_form, env)?;
    let form_idx = if test_eval.try_unwrap()?.is_truthy() {
        1
    } else {
        2
    };
    let Some(res_form) = args.get(form_idx) else {
        err!(format!("expected form idx ={}", form_idx))
    };
    eval.eval(res_form, env)
}

fn cond_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    for clause in args {
        let Exp::List(items) = clause else {
            err!("cond clause must be a list")
        };
        if items.is_empty() {
            err!("cond clause must not be empty")
        }

        let condition = &items[0];

        if let EvalFlow::Value(Exp::Bool(true)) = eval.eval(condition, env)? {
            return begin_impl(args, env, eval);
        }
    }
    Ok(Exp::Nil.value_flow())
}

fn and_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    for arg in args {
        let res = eval.eval(arg, env)?;
        if !res.try_unwrap()?.is_truthy() {
            return Ok(Exp::Bool(false).value_flow());
        }
    }
    Ok(Exp::Bool(true).value_flow())
}

fn or_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    for arg in args {
        let res = eval.eval(arg, env)?;
        if !res.try_unwrap()?.is_truthy() {
            return Ok(Exp::Bool(true).value_flow());
        }
    }
    Ok(Exp::Bool(false).value_flow())
}
