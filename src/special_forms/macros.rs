use crate::{
    error::SyntaxError,
    evaluator::Evaluator,
    expression::Exp,
    flow::EvalResult,
    lambda::LambdaExp,
    typedef::{Shared, SharedEnv},
};

pub fn register(eval: &Evaluator) {
    eval.register_special_form("define-macro", define_macro_impl);
}

fn define_macro_impl(args: &[Exp], env: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() < 3 {
        return Err(SyntaxError::invalid_args_size(
            "define-macro",
            3,
            args.len(),
        ));
    }

    let name = match &args[0] {
        Exp::Symbol(s) => s,
        _ => {
            return Err(SyntaxError::invalid_args_type_nth(
                "define-macro",
                "symbol",
                1,
            ));
        }
    };

    if !matches!(args[1], Exp::List(_) | Exp::DottedList(_, _)) {
        return Err(SyntaxError::invalid_args_type_nth(
            "define-macro",
            "list or dotted-list",
            2,
        ));
    }

    let lambda = LambdaExp::new(Shared::new(args[1].clone()), Vec::from(&args[2..]));
    Ok(env.define(name, Exp::Macro(lambda)).value_flow())
}
