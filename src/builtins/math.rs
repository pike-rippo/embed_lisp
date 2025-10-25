use crate::{
    error::{Error, Result},
    evaluator::Evaluator,
    expression::Exp,
    typedef::SharedEnv,
};

/// '+', '+', '-', '/'
pub fn register(env: &SharedEnv) {
    env.define("+", Exp::Function(add_impl));
    env.define("*", Exp::Function(mul_impl));
    env.define("-", Exp::Function(sub_impl));
    env.define("/", Exp::Function(div_impl));
    env.define("%", Exp::Function(mod_impl));
}

pub fn parse_list_of_floats(args: &[Exp]) -> Result<Vec<f64>> {
    args.iter().map(parse_single_float).collect()
}

fn parse_single_float(exp: &Exp) -> Result<f64> {
    match exp {
        Exp::Number(num) => Ok(*num),
        _ => Err(Error::from("expected a number")),
    }
}

fn add_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> Result<Exp> {
    let sum = parse_list_of_floats(args)?
        .iter()
        .fold(0.0, |sum, n| sum + n);
    Ok(Exp::Number(sum))
}

fn mul_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> Result<Exp> {
    let mul = parse_list_of_floats(args)?
        .iter()
        .fold(1.0, |mul, n| mul * n);
    Ok(Exp::Number(mul))
}

fn sub_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> Result<Exp> {
    let floats = parse_list_of_floats(args)?;
    let first = *floats
        .first()
        .ok_or(Error::from("expected at least one number"))?;
    let sum_of_rest = floats[1..].iter().fold(0.0, |sum, n| sum + n);
    Ok(Exp::Number(first - sum_of_rest))
}

fn div_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> Result<Exp> {
    let floats = parse_list_of_floats(args)?;
    let first = *floats
        .first()
        .ok_or(Error::from("expected at least one number"))?;
    let mul_of_rest = floats[1..].iter().fold(0.0, |mul, n| mul + n);
    Ok(Exp::Number(first / mul_of_rest))
}

fn mod_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> Result<Exp> {
    if args.len() != 2 {
        Err(Error::from("expected exactly two numbers"))
    } else {
        let floats = parse_list_of_floats(args)?;
        let first = *floats
            .first()
            .ok_or(Error::from("expected at least one number"))?;
        let second = *floats
            .get(1)
            .ok_or(Error::from("expected at least two numbers"))?;
        Ok(Exp::Number(first % second))
    }
}
