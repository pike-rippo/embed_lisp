use crate::{
    environment::SharedEnv,
    error::{Result, SyntaxError},
    evaluator::Evaluator,
    exp::Exp,
    flow::EvalResult,
};

pub fn register(env: &SharedEnv) {
    env.define("+", Exp::Primitive(add_impl));
    env.define("*", Exp::Primitive(mul_impl));
    env.define("-", Exp::Primitive(sub_impl));
    env.define("/", Exp::Primitive(div_impl));
    env.define("%", Exp::Primitive(mod_impl));
}

pub fn parse_list_of_floats(name: &str, args: &[Exp]) -> Result<Vec<f64>> {
    args.iter().map(|x| parse_single_float(name, x)).collect()
}

fn parse_single_float(name: &str, exp: &Exp) -> Result<f64> {
    match exp {
        Exp::Number(num) => Ok(*num),
        _ => Err(SyntaxError::invalid_args_type(name, "number")),
    }
}

fn add_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    let sum = parse_list_of_floats("+", args)?
        .iter()
        .fold(0.0, |sum, n| sum + n);
    Ok(Exp::Number(sum).value_flow())
}

fn mul_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    let mul = parse_list_of_floats("*", args)?
        .iter()
        .fold(1.0, |mul, n| mul * n);
    Ok(Exp::Number(mul).value_flow())
}

fn sub_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    let floats = parse_list_of_floats("-", args)?;
    let first = *floats
        .first()
        .ok_or(SyntaxError::not_enough_args("-", 2, args.len()))?;
    let sum_of_rest = floats[1..].iter().fold(0.0, |sum, n| sum + n);
    Ok(Exp::Number(first - sum_of_rest).value_flow())
}

fn div_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    let floats = parse_list_of_floats("/", args)?;
    let first = *floats
        .first()
        .ok_or(SyntaxError::not_enough_args("/", 2, args.len()))?;
    let mul_of_rest = floats[1..].iter().fold(0.0, |mul, n| mul + n);
    Ok(Exp::Number(first / mul_of_rest).value_flow())
}

fn mod_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        Err(SyntaxError::invalid_args_size("%", 2, args.len()))
    } else {
        let floats = parse_list_of_floats("%", args)?;
        let first = &floats[0];
        let second = &floats[1];
        Ok(Exp::Number(first % second).value_flow())
    }
}
