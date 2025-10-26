use crate::{
    err,
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
    if args.len() != 3 {
        err!("define-macro takes 3 arguments: name, args, and body")
    }

    let name = match &args[0] {
        Exp::Symbol(s) => s,
        _ => err!("first argument to define-macro must be a symbol"),
    };

    if !matches!(args[1], Exp::List(_) | Exp::DottedList(_, _)) {
        err!("define-macro expected args is list or dotted-list");
    }

    let lambda = LambdaExp::new(Shared::new(args[1].clone()), Shared::new(args[2].clone()));
    Ok(env.define(name, Exp::Macro(lambda)).value_flow())
}
