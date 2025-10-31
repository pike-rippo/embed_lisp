use crate::{
    error::SyntaxError,
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
    if args.len() != 3 {
        return Err(SyntaxError::invalid_args_size("if", 3, args.len()));
    }

    let idx = if eval.eval(&args[0], env)?.try_unwrap()?.is_truthy() {
        1
    } else {
        2
    };

    eval.eval(&args[idx], env)
}

fn cond_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    for clause in args {
        let Exp::List(items) = clause else {
            return Err(SyntaxError::reason("cond clause must be a list"));
        };
        if items.is_empty() {
            return Err(SyntaxError::reason("cond clause must not be empty"));
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
