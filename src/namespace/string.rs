use crate::{
    Error, Evaluator, Exp,
    environment::SharedEnv,
    error::{Result, SyntaxError},
    exp::NameSpace,
    flow::EvalResult,
    typedef::Shared,
};

pub fn register(env: &SharedEnv) {
    let ns = NameSpace::default();

    ns.define("number", Exp::Primitive(number_impl));
    ns.define("symbol", Exp::Primitive(symbol_impl));
    ns.define("string", Exp::Primitive(string_impl));
    ns.define("concat", Exp::Primitive(concat_impl));
    ns.define("substr", Exp::Primitive(substr_impl));
    ns.define("char-at", Exp::Primitive(char_at_impl));
    ns.define("index-of", Exp::Primitive(index_of_impl));
    ns.define("starts-with?", Exp::Primitive(starts_with_impl));
    ns.define("ends-with?", Exp::Primitive(ends_with_impl));
    ns.define("contains?", Exp::Primitive(contains_impl));
    ns.define("replace", Exp::Primitive(replace_impl));
    ns.define("split", Exp::Primitive(split_impl));
    ns.define("join", Exp::Primitive(join_impl));

    ns.define("trim", Exp::Primitive(trim_impl));
    ns.define("to-upper", Exp::Primitive(to_upper_impl));
    ns.define("to-lower", Exp::Primitive(to_lower_impl));
    ns.define("repeat", Exp::Primitive(repeat_impl));

    env.assign("str", Exp::Namespace(Shared::new(ns)));
}

fn extract_string<'a>(func_name: &'static str, e: &'a Exp) -> Result<&'a String> {
    match e {
        Exp::String(s) => Ok(s),
        _ => Err(SyntaxError::invalid_args_type(func_name, "string")),
    }
}

fn number_impl(args: &[Exp], _env: &SharedEnv, _eval: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("number", 1, args.len()));
    }
    let Exp::String(s) = &args[0] else {
        return Err(SyntaxError::invalid_args_type("number", "string"));
    };
    let potential_float: std::result::Result<f64, std::num::ParseFloatError> = s.parse();
    match potential_float {
        Ok(v) => Ok(Exp::Number(v).value_flow()),
        Err(_) => Err(Error::Reason(format!("'{}' is not number", s))),
    }
}

fn symbol_impl(args: &[Exp], _env: &SharedEnv, _eval: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("symbol", 1, args.len()));
    }

    let Exp::String(s) = &args[0] else {
        return Err(SyntaxError::invalid_args_type("symbol", "string"));
    };

    Ok(Exp::Symbol(s.clone()).value_flow())
}

fn string_impl(args: &[Exp], env: &SharedEnv, eval: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("string", 1, args.len()));
    }

    Ok(Exp::String(format!("{}", eval.eval(&args[0], env)?.try_unwrap()?)).value_flow())
}

fn concat_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    let size = args
        .iter()
        .map(|e| extract_string("concat", e))
        .try_fold(0, |acc, r| {
            let s = r?;
            Ok::<usize, Error>(acc + s.len())
        })?;
    let s = args.iter().map(|e| extract_string("concat", e)).try_fold(
        String::with_capacity(size),
        |mut acc, r| {
            let s = r?;
            acc.push_str(s);
            Ok::<String, Error>(acc)
        },
    )?;

    Ok(Exp::String(s).value_flow())
}

fn substr_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() < 2 {
        return Err(SyntaxError::not_enough_args("substr", 2, args.len()));
    }

    let Exp::String(s) = &args[0] else {
        return Err(SyntaxError::invalid_args_type_nth("substr", "string", 1));
    };

    let Exp::Number(start) = &args[1] else {
        return Err(SyntaxError::invalid_args_type_nth("substr", "number", 2));
    };

    if args.len() == 2 {
        if *start as usize >= s.len() {
            return Err(Error::reason("string index out of range"));
        }
        Ok(Exp::String(s[*start as usize..].to_string()).value_flow())
    } else {
        let Exp::Number(end) = args[2] else {
            return Err(SyntaxError::invalid_args_type_nth("substr", "number", 3));
        };
        if *start as usize >= s.len() || end as usize >= s.len() {
            return Err(Error::reason("string index out of range"));
        }

        Ok(Exp::String(s[*start as usize..end as usize].to_string()).value_flow())
    }
}

fn char_at_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        return Err(SyntaxError::invalid_args_size("char-at", 2, args.len()));
    }

    let Exp::String(s) = &args[0] else {
        return Err(SyntaxError::invalid_args_type_nth("char-at", "string", 1));
    };

    let Exp::Number(index) = &args[1] else {
        return Err(SyntaxError::invalid_args_type_nth("char-at", "number", 2));
    };

    if *index as usize >= s.len() {
        return Err(Error::reason("string index out of range"));
    }

    Ok(Exp::String(s.chars().nth(*index as usize).unwrap().to_string()).value_flow())
}
fn index_of_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        return Err(SyntaxError::invalid_args_size("index-of", 2, args.len()));
    }

    let Exp::String(s) = &args[0] else {
        return Err(SyntaxError::invalid_args_type_nth("index-of", "string", 1));
    };

    let Exp::String(sub) = &args[1] else {
        return Err(SyntaxError::invalid_args_type_nth("index-of", "string", 2));
    };

    let mut matches = s.match_indices(sub);
    if let Some((index, _)) = matches.next() {
        Ok(Exp::Number(index as f64).value_flow())
    } else {
        Ok(Exp::Number(-1.0).value_flow())
    }
}
fn starts_with_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        return Err(SyntaxError::invalid_args_size(
            "starts-with?",
            2,
            args.len(),
        ));
    }

    let Exp::String(s) = &args[0] else {
        return Err(SyntaxError::invalid_args_type_nth(
            "starts-with?",
            "string",
            1,
        ));
    };

    let Exp::String(sub) = &args[1] else {
        return Err(SyntaxError::invalid_args_type_nth(
            "starts-with?",
            "string",
            2,
        ));
    };

    Ok(Exp::Bool(s.starts_with(sub)).value_flow())
}
fn ends_with_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        return Err(SyntaxError::invalid_args_size("ends-with?", 2, args.len()));
    }

    let Exp::String(s) = &args[0] else {
        return Err(SyntaxError::invalid_args_type_nth(
            "ends-with?",
            "string",
            1,
        ));
    };

    let Exp::String(sub) = &args[1] else {
        return Err(SyntaxError::invalid_args_type_nth(
            "ends-with?",
            "string",
            2,
        ));
    };

    Ok(Exp::Bool(s.ends_with(sub)).value_flow())
}
fn contains_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        return Err(SyntaxError::invalid_args_size("contains?", 2, args.len()));
    }

    let Exp::String(s) = &args[0] else {
        return Err(SyntaxError::invalid_args_type_nth("contains?", "string", 1));
    };

    let Exp::String(sub) = &args[1] else {
        return Err(SyntaxError::invalid_args_type_nth("contains?", "string", 2));
    };

    Ok(Exp::Bool(s.contains(sub)).value_flow())
}
fn replace_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 3 {
        return Err(SyntaxError::invalid_args_size("replace", 2, args.len()));
    }

    let Exp::String(s) = &args[0] else {
        return Err(SyntaxError::invalid_args_type_nth("replace", "string", 1));
    };

    let Exp::String(from) = &args[1] else {
        return Err(SyntaxError::invalid_args_type_nth("replace", "string", 2));
    };

    let Exp::String(to) = &args[2] else {
        return Err(SyntaxError::invalid_args_type_nth("replace", "string", 3));
    };

    Ok(Exp::String(s.replace(from, to)).value_flow())
}
fn split_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        return Err(SyntaxError::invalid_args_size("split", 2, args.len()));
    }

    let Exp::String(s) = &args[0] else {
        return Err(SyntaxError::invalid_args_type_nth("split", "string", 1));
    };

    let Exp::String(delim) = &args[1] else {
        return Err(SyntaxError::invalid_args_type_nth("split", "string", 2));
    };

    Ok(Exp::List(
        s.split(delim)
            .map(|s| Exp::String(s.to_string()))
            .collect::<Vec<Exp>>(),
    )
    .value_flow())
}
fn join_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        return Err(SyntaxError::invalid_args_size("join", 2, args.len()));
    }

    let Exp::List(list) = &args[0] else {
        return Err(SyntaxError::invalid_args_type_nth("join", "list", 1));
    };

    let Exp::String(sep) = &args[1] else {
        return Err(SyntaxError::invalid_args_type_nth("join", "string", 2));
    };

    let strings = list.iter().map(|e| extract_string("join", e)).try_fold(
        Vec::with_capacity(list.len()),
        |mut acc, r| {
            let s = r?;
            acc.push(s.to_owned());
            Ok::<Vec<String>, Error>(acc)
        },
    )?;

    Ok(Exp::String(strings.join(sep)).value_flow())
}
fn trim_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("trim", 1, args.len()));
    }

    let Exp::String(s) = &args[0] else {
        return Err(SyntaxError::invalid_args_type_nth("trim", "string", 1));
    };

    Ok(Exp::String(s.trim().to_string()).value_flow())
}
fn to_upper_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("to_upper", 1, args.len()));
    }

    let Exp::String(s) = &args[0] else {
        return Err(SyntaxError::invalid_args_type_nth("to_upper", "string", 1));
    };

    Ok(Exp::String(s.to_uppercase()).value_flow())
}
fn to_lower_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 1 {
        return Err(SyntaxError::invalid_args_size("to_lower", 1, args.len()));
    }

    let Exp::String(s) = &args[0] else {
        return Err(SyntaxError::invalid_args_type_nth("to_lower", "string", 1));
    };

    Ok(Exp::String(s.to_lowercase()).value_flow())
}
fn repeat_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    if args.len() != 2 {
        return Err(SyntaxError::invalid_args_size("repeat", 2, args.len()));
    }

    let Exp::String(s) = &args[0] else {
        return Err(SyntaxError::invalid_args_type_nth("repeat", "string", 1));
    };

    let Exp::Number(n) = &args[1] else {
        return Err(SyntaxError::invalid_args_type_nth("repeat", "number", 2));
    };

    Ok(Exp::String(s.repeat(*n as usize)).value_flow())
}
