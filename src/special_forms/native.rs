use crate::{Evaluator, Exp, error::Result, typedef::SharedEnv};

pub fn register(eval: &Evaluator) {
    eval.register_special_form("create-native", create_native_impl);
}

fn create_native_impl(args: &[Exp], _: &SharedEnv, eval: &Evaluator) -> Result<Exp> {
    let (name, args) = args
        .split_first()
        .ok_or("create-native expected at least one argument")?;

    let Exp::String(name) = name else {
        return Err("create-native expected a string as the first argument".into());
    };

    eval.create_native_object(&name, args)
}
