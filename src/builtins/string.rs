use crate::{
    Error, Evaluator, Exp,
    environment::SharedEnv,
    error::{Result, SyntaxError},
    flow::EvalResult,
};

pub fn register(env: &SharedEnv) {
    env.define("concat", Exp::Function(concat_impl));
    env.define("substr", Exp::Function(substr_impl));
    env.define("char-at", Exp::Function(char_at_impl));
    env.define("index-of", Exp::Function(index_of_impl));
    env.define("starts-with?", Exp::Function(starts_with_impl));
    env.define("ends-with?", Exp::Function(ends_with_impl));
    env.define("contains?", Exp::Function(contains_impl));
    env.define("replace", Exp::Function(replace_impl));
    env.define("split", Exp::Function(split_impl));
    env.define("join", Exp::Function(join_impl));

    env.define("trim", Exp::Function(trim_impl));
    env.define("to-upper", Exp::Function(to_upper_impl));
    env.define("to-lower", Exp::Function(to_lower_impl));
    env.define("repeat", Exp::Function(repeat_impl));
    // env.define("pad-left", Exp::Function(pad_left_impl));
    // env.define("pad-right", Exp::Function(pad_right_impl));

    // env.define("format", Exp::Function(format_impl));
    // env.define("escape", Exp::Function(escape_impl));
    // env.define("unescape", Exp::Function(unescape_impl));
}

fn extract_string<'a>(func_name: &'static str, e: &'a Exp) -> Result<&'a String> {
    match e {
        Exp::String(s) => Ok(s),
        _ => Err(SyntaxError::invalid_args_type(func_name, "string")),
    }
}

fn concat_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
    // let mut iter = args.iter().map(|e| extract_string("concat", e));
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
// fn pad_left_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
//     panic!()
// }
// fn pad_right_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
//     panic!()
// }
// fn format_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
//     if args.is_empty() {
//         return Err(SyntaxError::invalid_args_size("format", 1, args.len()));
//     }

//     let Exp::String(format) = &args[0] else {
//         return Err(SyntaxError::invalid_args_type_nth("format", "string", 1));
//     };

//     let args = format_args!(format, )

//     Ok(Exp::String(format).value_flow())
// }
// fn escape_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
//     panic!()
// }
// fn unescape_impl(args: &[Exp], _: &SharedEnv, _: &Evaluator) -> EvalResult {
//     panic!()
// }
