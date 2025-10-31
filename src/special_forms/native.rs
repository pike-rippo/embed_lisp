use crate::{Evaluator, Exp, error::SyntaxError, flow::EvalResult, typedef::SharedEnv};

pub fn register(eval: &Evaluator) {
    eval.register_special_form("create-native", create_native_impl);
}

fn create_native_impl(args: &[Exp], _: &SharedEnv, eval: &Evaluator) -> EvalResult {
    let (name, args) = args
        .split_first()
        // .ok_or("create-native expected at least one argument")?;
        .ok_or(SyntaxError::not_enough_args("create-native", 2, args.len()))?;

    let Exp::String(name) = name else {
        // return Err("create-native expected a string as the first argument".into());
        return Err(SyntaxError::invalid_args_type_nth(
            "create-native",
            "string",
            1,
        ));
    };

    eval.create_native_object(name, args)
}
