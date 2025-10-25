use crate::{
    err, error::Result, evaluator::Evaluator, expression::Exp, special_forms::core::begin_impl,
    typedef::SharedEnv,
};

/// 'if', 'cond', 'and', 'or'
pub fn register(eval: &Evaluator) {
    eval.register_special_form("if", if_impl);
    eval.register_special_form("cond", cond_impl);
    eval.register_special_form("and", and_impl);
    eval.register_special_form("or", or_impl);
}

fn if_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> Result<Exp> {
    let Some(test_form) = args.first() else {
        err!("expected test form")
    };
    let test_eval = eval.eval(test_form, env)?;
    let form_idx = if test_eval.is_truthy() { 1 } else { 2 };
    let Some(res_form) = args.get(form_idx) else {
        err!(format!("expected form idx ={}", form_idx))
    };
    eval.eval(res_form, env)
}

fn cond_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> Result<Exp> {
    for clause in args {
        let Exp::List(items) = clause else {
            err!("cond clause must be a list")
        };
        if items.is_empty() {
            err!("cond clause must not be empty")
        }

        let condition = &items[0];

        if let Exp::Bool(true) = eval.eval(condition, env)? {
            return begin_impl(args, env, eval);
        }
    }
    Ok(Exp::Nil)
}

fn and_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> Result<Exp> {
    for arg in args {
        let res = eval.eval(arg, env)?;
        if !res.is_truthy() {
            return Ok(Exp::Bool(false));
        }
    }
    Ok(Exp::Bool(true))
}

fn or_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> Result<Exp> {
    for arg in args {
        let res = eval.eval(arg, env)?;
        if !res.is_truthy() {
            return Ok(Exp::Bool(true));
        }
    }
    Ok(Exp::Bool(false))
}
