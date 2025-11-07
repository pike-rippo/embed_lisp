use std::vec;

use crate::{
    environment::SharedEnv,
    error::{Error, SyntaxError},
    evaluator::Evaluator,
    exp::Exp,
    flow::EvalResult,
};

// Exp::Primitive(|args: &[Exp], _: &SharedEnv, _: &Evaluator| -> EvalResult {})

/// 'car', 'cdr', 'cons', 'list', 'append', 'length', 'apply'
pub fn register(env: &SharedEnv) {
    env.define("car", Exp::Primitive(car_impl));
    env.define("cdr", Exp::Primitive(cdr_impl));
    env.define("cons", Exp::Primitive(cons_impl));
    env.define("list", Exp::Primitive(list_impl));
    env.define("append", Exp::Primitive(append_impl));
    env.define("length", Exp::Primitive(length_impl));
    env.define("funcall", Exp::Primitive(funcall_impl));
    env.define("apply", Exp::Primitive(apply_impl));
}

fn car_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("car", 1, args.len()));
    }
    match &args[0] {
        Exp::List(list) => Ok(list.first().cloned().unwrap_or(Exp::Nil).value_flow()),
        _ => return Err(SyntaxError::invalid_args_type("car", "list")),
    }
}

fn cdr_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("cdr", 1, args.len()));
    }
    match &args[0] {
        Exp::List(list) => {
            let (_, res) = list
                .split_first()
                .ok_or(Error::reason("could not split first"))?;
            Ok(Exp::List(res.to_vec()).value_flow())
        }
        _ => return Err(SyntaxError::invalid_args_type("cdr", "list")),
    }
}

fn cons_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        return Err(SyntaxError::invalid_args_size("cons", 2, args.len()));
    }

    let head = &args[0];
    match &args[1] {
        Exp::Nil => Ok(Exp::List(vec![head.clone()]).value_flow()),
        Exp::List(list) => {
            let mut new_list = vec![head.clone()];
            new_list.extend_from_slice(list);
            Ok(Exp::List(new_list).value_flow())
        }
        _ => Ok(Exp::List(vec![head.clone(), args[1].clone()]).value_flow()),
    }
}

fn list_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    let mut new_list = Vec::with_capacity(args.len());
    for exp in args {
        new_list.push(exp.unwrap_quote());
    }

    Ok(Exp::List(new_list).value_flow())
}

fn append_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        return Err(SyntaxError::invalid_args_size("append", 2, args.len()));
    }

    match &args[0] {
        Exp::Nil => Ok(args[1].clone().value_flow()),
        Exp::List(l0) => match &args[1] {
            Exp::List(l1) => {
                let list: Vec<Exp> = l0.iter().cloned().chain(l1.iter().cloned()).collect();
                Ok(Exp::List(list).value_flow())
            }
            exp => {
                let mut list = l0.clone();
                list.push(exp.clone());
                Ok(Exp::List(list).value_flow())
            }
        },
        _ => return Err(SyntaxError::invalid_args_type("append", "list")),
    }
}

fn length_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("length", 1, args.len()));
    }

    match &args[0] {
        Exp::List(l) => Ok(Exp::Number(l.len() as f64).value_flow()),
        Exp::String(s) => Ok(Exp::Number(s.len() as f64).value_flow()),
        // _ => err!("length expects a list or string"),
        _ => {
            return Err(SyntaxError::invalid_args_type("length", "list or string"));
        }
    }
}

fn funcall_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    if args.is_empty() {
        return Err(SyntaxError::not_enough_args("funcall", 1, args.len()));
    }

    let f = &args[0];
    let rest_args = &args[1..];

    eval.apply(f.clone(), rest_args, env)
}

fn apply_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    if args.len() < 2 {
        return Err(SyntaxError::not_enough_args("apply", 2, args.len()));
    }
    let f = &args[0];
    let Exp::List(list_arg) = &args[args.len() - 1] else {
        return Err(SyntaxError::invalid_args_type_nth(
            "apply",
            "list",
            args.len(),
        ));
    };
    if args.len() == 2 {
        eval.apply(f.clone(), list_arg, env)
    } else {
        let mut arg_forms: Vec<Exp> = Vec::with_capacity(args.len() + list_arg.len() - 1);
        arg_forms.extend_from_slice(&args[1..args.len() - 1]);
        arg_forms.extend_from_slice(list_arg);
        eval.apply(f.clone(), &arg_forms, env)
    }
}
