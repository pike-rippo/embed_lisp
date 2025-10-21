use std::vec;

use crate::{
    environment::EnvRc,
    err,
    error::{Error, Result},
    evaluator::Evaluator,
    expression::Exp,
    ok,
};

// Exp::Function(|args: &[Exp], _: &EnvRc, _: &Evaluator| -> Result<Exp> {})

/// 'car', 'cdr', 'cons', 'list', 'append', 'length', 'apply'
pub fn register(env: &EnvRc) {
    env.define("car", Exp::Function(car_impl));
    env.define("cdr", Exp::Function(cdr_impl));
    env.define("cons", Exp::Function(cons_impl));
    env.define("list", Exp::Function(list_impl));
    env.define("append", Exp::Function(append_impl));
    env.define("length", Exp::Function(length_impl));
    env.define("apply", Exp::Function(apply_impl));
}

fn car_impl(args: &[Exp], _: &EnvRc, _: &Evaluator) -> Result<Exp> {
    if args.len() != 1 {
        err!("car expected exactly one argument")
    }
    match &args[0] {
        Exp::List(list) => Ok(list.first().cloned().unwrap_or(Exp::Nil)),
        _ => err!("expected a list"),
    }
}

fn cdr_impl(args: &[Exp], _: &EnvRc, _: &Evaluator) -> Result<Exp> {
    if args.len() != 1 {
        err!("cdr expected exactly one argument")
    }
    match &args[0] {
        Exp::List(list) => {
            let (_, res) = list
                .split_first()
                .ok_or(Error::from("could not split first"))?;
            ok!(res.to_vec())
        }
        _ => err!("expected a list"),
    }
}

fn cons_impl(args: &[Exp], _: &EnvRc, _: &Evaluator) -> Result<Exp> {
    if args.len() != 2 {
        err!("cons expected exactly two arguments")
    }

    let head = &args[0];
    match &args[1] {
        Exp::Nil => Ok(Exp::List(vec![head.clone()])),
        Exp::List(list) => {
            let mut new_list = vec![head.clone()];
            new_list.extend_from_slice(&list);
            Ok(Exp::List(new_list))
        }
        _ => Ok(Exp::List(vec![head.clone(), args[1].clone()])),
    }
}

fn list_impl(args: &[Exp], _: &EnvRc, _: &Evaluator) -> Result<Exp> {
    let mut new_list = Vec::with_capacity(args.len());
    for exp in args {
        new_list.push(exp.unwrap_quote());
    }

    ok!(new_list)
}

fn append_impl(args: &[Exp], _: &EnvRc, _: &Evaluator) -> Result<Exp> {
    if args.len() != 2 {
        err!("append expected exactly two arguments")
    }

    match &args[0] {
        Exp::Nil => return Ok(args[1].clone()),
        Exp::List(l0) => match &args[1] {
            Exp::List(l1) => {
                let list: Vec<Exp> = l0.iter().cloned().chain(l1.iter().cloned()).collect();
                ok!(list)
            }
            exp => {
                let mut list = l0.clone();
                list.push(exp.clone());
                ok!(list)
            }
        },
        _ => err!("append expected a list"),
    }
}

fn length_impl(args: &[Exp], _: &EnvRc, _: &Evaluator) -> Result<Exp> {
    if args.len() != 1 {
        err!("length takes one argument")
    }

    match &args[0] {
        Exp::List(l) => ok!(l.len() as f64),
        Exp::String(s) => ok!(s.len() as f64),
        _ => err!("length expects a list or string"),
    }
}

fn apply_impl(args: &[Exp], env: &EnvRc, eval: &Evaluator) -> Result<Exp> {
    if args.len() < 2 {
        err!("apply takes at least 2 arguments")
    }
    let f = &args[0];
    let Exp::List(list_arg) = &args[args.len() - 1] else {
        err!("last argument of apply must be list")
    };
    if args.len() == 2 {
        eval.apply(f.clone(), list_arg, env)
    } else {
        let mut arg_forms: Vec<Exp> = Vec::with_capacity(args.len() + list_arg.len() - 1);
        arg_forms.extend_from_slice(&args[1..args.len() - 1]);
        arg_forms.extend_from_slice(&list_arg);
        eval.apply(f.clone(), &arg_forms, env)
    }
}
