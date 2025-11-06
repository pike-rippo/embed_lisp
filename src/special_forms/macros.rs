use crate::{
    environment::SharedEnv,
    error::SyntaxError,
    evaluator::Evaluator,
    exp::{Exp, LambdaExp},
    flow::EvalResult,
    typedef::Shared,
};

pub fn register(eval: &Evaluator) {
    eval.register_special_form("create-macro", create_macro_impl);
    eval.register_special_form("define-macro", define_macro_impl);
}

fn create_macro_impl(args: &[Exp], env: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() < 2 {
        return Err(SyntaxError::invalid_args_size(
            "create-macro",
            2,
            args.len(),
        ));
    }

    // let name = match &args[0] {
    //     Exp::Symbol(s) => s,
    //     _ => {
    //         return Err(SyntaxError::invalid_args_type_nth(
    //             "define-macro",
    //             "symbol",
    //             1,
    //         ));
    //     }
    // };

    if !matches!(args[0], Exp::List(_) | Exp::DottedList(_, _)) {
        return Err(SyntaxError::invalid_args_type_nth(
            "create-macro",
            "list or dotted-list",
            1,
        ));
    }

    let lambda = LambdaExp::new(Shared::new(args[0].clone()), Vec::from(&args[1..]));
    Ok(Exp::Macro(lambda).value_flow())
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
