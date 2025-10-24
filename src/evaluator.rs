use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, AtomicUsize},
    },
};

use crate::{
    GENESYM_COUNTER,
    environment::Env,
    err,
    error::{Error, Result},
    expression::Exp,
    lambda::LambdaExp,
    ok, read_expect, special_forms,
    typedef::{Shared, SharedEnv},
    write_error, write_expect,
};

pub type SpecialFormFn = fn(&[Exp], &SharedEnv, &Evaluator) -> Result<Exp>;

pub struct Evaluator {
    special_forms: RwLock<HashMap<String, SpecialFormFn>>,
    trace: AtomicBool,
    gensym_counter: Arc<&'static AtomicUsize>,
}

#[cfg(feature = "async")]
impl Clone for Evaluator {
    fn clone(&self) -> Self {
        self.deep_copy()
    }
}

impl Evaluator {
    pub fn new() -> Self {
        let eval = Self {
            special_forms: RwLock::new(HashMap::new()),
            trace: AtomicBool::new(false),
            gensym_counter: Arc::new(&GENESYM_COUNTER),
        };

        special_forms::register_all_special_form(&eval);

        eval
    }

    #[cfg(feature = "async")]
    pub fn deep_copy(&self) -> Self {
        use std::sync::RwLock;

        Self {
            special_forms: RwLock::new(self.special_forms.read().expect(read_expect!()).clone()),
            trace: AtomicBool::new(self.trace.load(std::sync::atomic::Ordering::Relaxed)),
            gensym_counter: Arc::new(&GENESYM_COUNTER),
        }
    }

    pub fn register_special_form(&self, k: &str, f: SpecialFormFn) {
        self.special_forms
            .write()
            .expect(write_expect!())
            .insert(k.to_string(), f);
    }

    pub fn get_gensym_id(&self) -> usize {
        self.gensym_counter
            .fetch_add(1, std::sync::atomic::Ordering::Release)
    }

    pub fn set_trace(&self, value: bool) {
        self.trace
            .store(value, std::sync::atomic::Ordering::Release);
    }

    pub fn eval(&self, exp: &Exp, env: &SharedEnv) -> Result<Exp> {
        if self.trace.load(std::sync::atomic::Ordering::Relaxed) {
            println!("trace eval: {}", exp);
        }

        match exp {
            Exp::Nil => Ok(exp.clone()),
            Exp::Number(_) => Ok(exp.clone()),
            Exp::Bool(_) => Ok(exp.clone()),
            Exp::String(_) => Ok(exp.clone()),
            Exp::Native(_) => Ok(exp.clone()),
            Exp::Function(_) => Err(Error::from("unexpected form: Function")),
            Exp::Lambda(_) => err!("unexpected form: lambda"),
            Exp::Macro(_) => err!("unexpected form: macro"),
            Exp::Symbol(k) => env
                .lookup(k)
                .ok_or(Error::Reason(format!("unexpected symbol '{}'", k))),
            Exp::List(list) => {
                let Some(first_form) = list.first() else {
                    ok!(Exp::Nil);
                };
                let args = &list[1..];
                if let Exp::Symbol(k) = first_form {
                    if let Some(f) = self.special_forms.read().expect(read_expect!()).get(k) {
                        return f(args, env, self);
                    }
                }

                let first_eval = self.eval(first_form, env)?;
                self.apply(first_eval, &args, env)
            }
            #[cfg(feature = "async")]
            Exp::Future(_) => err!("unexpected form: future"),
            #[cfg(feature = "async")]
            Exp::Task(future) => future.get(),
        }
    }

    pub fn apply(&self, exp: Exp, args: &[Exp], env: &SharedEnv) -> Result<Exp> {
        match exp {
            Exp::Function(f) => f(&self.eval_form(args, env)?, env, self),
            Exp::Lambda(lambda) => self.apply_lambda(lambda, args, env),
            Exp::Macro(lambda) => self.apply_macro(lambda, args, env),
            _ => err!("first form must be function, lambda or macro"),
        }
    }

    pub fn eval_form(&self, args: &[Exp], env: &SharedEnv) -> Result<Vec<Exp>> {
        args.iter().map(|x| self.eval(x, env)).collect()
    }

    fn apply_lambda(&self, lambda: LambdaExp, args: &[Exp], env: &SharedEnv) -> Result<Exp> {
        let keys = parse_list_of_symbol_strings(&lambda.params_exp)?;
        if keys.len() != args.len() {
            err!(format!(
                "expected {} arguments, got {}",
                keys.len(),
                args.len()
            ))
        }
        let values = self.eval_form(args, env)?;
        let child = Env::extend(env.clone(), &keys, &values);
        self.eval(&lambda.body_exp, &child)
    }

    fn apply_macro(&self, lambda: LambdaExp, args: &[Exp], env: &SharedEnv) -> Result<Exp> {
        let expanded = self.expand_macro(lambda, args, env)?;
        self.eval(&expanded, &env)
    }

    pub fn expand_macro(&self, lambda: LambdaExp, args: &[Exp], env: &SharedEnv) -> Result<Exp> {
        let keys = parse_list_of_symbol_strings(&lambda.params_exp)?;
        if keys.len() != args.len() {
            err!(format!(
                "expected {} arguments, got {}",
                keys.len(),
                args.len()
            ))
        }
        let child = Env::extend(env.clone(), &keys, &args);
        self.eval(&lambda.body_exp, &child)
        // self.eval_quasiquote(&lambda.body_exp, &child)
    }

    // pub fn eval_quasiquote(&self, exp: &Exp, env: &SharedEnv) -> Result<Exp> {
    //     let Exp::List(list) = exp else {
    //         return Ok(exp.clone());
    //     };
    //     let mut result = Vec::new();
    //     for item in list {
    //         match item {
    //             Exp::Symbol(s) if s == "quasiquote" => continue,
    //             Exp::List(inner) if !inner.is_empty() => match &inner[0] {
    //                 Exp::Symbol(s) if s == "unquote" => {
    //                     if inner.len() != 2 {
    //                         err!("unquote expects one argument")
    //                     }
    //                     let evaluated = self.eval(&inner[1], env)?;
    //                     result.push(evaluated);
    //                 }
    //                 Exp::Symbol(s) if s == "unquote-splicing" => {
    //                     if inner.len() != 2 {
    //                         err!("unquote-splicing expects one argument")
    //                     }
    //                     let evaluated = self.eval(&inner[1], env)?;
    //                     if let Exp::List(splice_items) = evaluated {
    //                         result.extend(splice_items);
    //                     } else {
    //                         err!("unquote-splicing requires list result")
    //                     }
    //                 }
    //                 _ => {
    //                     let quoted = self.eval_quasiquote(item, env)?;
    //                     result.push(quoted);
    //                 }
    //             },
    //             _ => {
    //                 let quoted = self.eval_quasiquote(item, env)?;
    //                 result.push(quoted);
    //             }
    //         }
    //     }

    //     ok!(result)
    // }
    pub fn eval_quasiquote(&self, exp: &Exp, env: &SharedEnv) -> Result<Exp> {
        match exp {
            Exp::List(list) if !list.is_empty() => {
                match &list[0] {
                    Exp::Symbol(s) if s == "quasiquote" => {
                        if list.len() != 2 {
                            err!("quasiquote expects one argument")
                        }
                        self.eval_quasiquote(&list[1], env)
                    }
                    Exp::Symbol(s) if s == "unquote" => {
                        // unquote のみ評価
                        self.eval(&list[1], env)
                    }
                    Exp::Symbol(s) if s == "unquote-splicing" => {
                        // splicing も同様
                        let val = self.eval(&list[1], env)?;
                        if let Exp::List(items) = val {
                            Ok(Exp::List(items)) // ここではまだリスト化
                        } else {
                            err!("unquote-splicing requires list")
                        }
                    }
                    _ => {
                        // 再帰展開
                        let mut new_list = Vec::new();
                        for item in list {
                            new_list.push(self.eval_quasiquote(item, env)?);
                        }
                        Ok(Exp::List(new_list))
                    }
                }
            }
            _ => Ok(exp.clone()), // シンボルや数値はそのまま
        }
    }
}

pub fn parse_list_of_symbol_strings(form: &Shared<Exp>) -> Result<Vec<String>> {
    match form.as_ref() {
        Exp::List(list) => list
            .iter()
            .map(|x| match x {
                Exp::Symbol(s) => Ok(s.clone()),
                _ => err!("expected symbol in the argument list"),
            })
            .collect(),
        _ => err!("expected args form to be a list"),
    }
}

// use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::atomic::AtomicBool};

// use crate::{
//     environment::Env,
//     err,
//     error::{Error, Result},
//     expression::Exp,
//     lambda::LambdaExp,
//     ok, special_forms,
//     typedef::{Shared, SharedEnv, SharedMut},
// };

// pub type SpecialFormFn = fn(&[Exp], &SharedEnv, &Evaluator) -> Result<Exp>;

// pub struct Evaluator {
//     special_forms: SharedMut<HashMap<String, SpecialFormFn>>,
//     trace: AtomicBool,
// }

// #[cfg(feature = "async")]
// impl Clone for Evaluator {
//     fn clone(&self) -> Self {
//         self.deep_copy()
//     }
// }

// impl Evaluator {
//     pub fn new() -> Self {
//         let eval = Self {
//             special_forms: SharedMut::new(HashMap::new()),
//             trace: AtomicBool::new(false),
//         };

//         special_forms::register_all_special_form(&eval);

//         eval
//     }

//     #[cfg(feature = "async")]
//     pub fn deep_copy(&self) -> Self {
//         Self {
//             special_forms: SharedMut::new(self.special_forms.blocking_lock().clone()),
//             trace: AtomicBool::new(self.trace.load(std::sync::atomic::Ordering::Relaxed)),
//         }
//     }

//     pub fn register_special_form(&self, k: &str, f: SpecialFormFn) {
//         #[cfg(not(feature = "async"))]
//         self.special_forms.borrow_mut().insert(k.to_string(), f);

//         #[cfg(feature = "async")]
//         self.special_forms.blocking_lock().insert(k.to_string(), f);
//     }

//     pub fn set_trace(&self, value: bool) {
//         self.trace
//             .store(value, std::sync::atomic::Ordering::Release);
//     }

//     pub fn eval(&self, exp: &Exp, env: &SharedEnv) -> Result<Exp> {
//         if self.trace.load(std::sync::atomic::Ordering::Relaxed) {
//             println!("trace eval: {}", exp);
//         }

//         match exp {
//             Exp::Nil => Ok(exp.clone()),
//             Exp::Number(_) => Ok(exp.clone()),
//             Exp::Bool(_) => Ok(exp.clone()),
//             Exp::String(_) => Ok(exp.clone()),
//             Exp::Native(_) => Ok(exp.clone()),
//             // Exp::Future(future) => future.get(),
//             Exp::Function(_) => Err(Error::from("unexpected form: Function")),
//             Exp::Lambda(_) => err!("unexpected form: lambda"),
//             Exp::Macro(_) => err!("unexpected form: macro"),
//             Exp::Symbol(k) => env
//                 .lookup(k)
//                 .ok_or(Error::Reason(format!("unexpected symbol '{}'", k))),
//             Exp::List(list) => {
//                 let Some(first_form) = list.first() else {
//                     ok!(Exp::Nil);
//                 };
//                 let args = &list[1..];
//                 if let Exp::Symbol(k) = first_form {
//                     // #[cfg(not(feature = "async"))]
//                     // if let Some(f) = self.special_forms.borrow().get(k) {
//                     //     return f(args, env, self);
//                     // }

//                     // #[cfg(feature = "async")]
//                     // if let Some(f) = self.special_forms.blocking_lock().get(k) {
//                     //     return f(args, env, self);
//                     // }

//                     let f_opt = {
//                         #[cfg(not(feature = "async"))]
//                         {
//                             self.special_forms.borrow().get(k).cloned()
//                         }

//                         #[cfg(feature = "async")]
//                         if tokio::runtime::Handle::try_current().is_ok() {
//                             tokio::task::block_in_place(|| {
//                                 self.special_forms.blocking_lock().get(k).cloned()
//                             })
//                         } else {
//                             self.special_forms.blocking_lock().get(k).cloned()
//                         }

//                         // {
//                         //     self.special_forms.blocking_lock().get(k).cloned()
//                         // }
//                     };

//                     if let Some(f) = f_opt {
//                         return f(args, env, self);
//                     }
//                 }

//                 let first_eval = self.eval(first_form, env)?;
//                 self.apply(first_eval, &args, env)
//             }
//             #[cfg(feature = "async")]
//             Exp::Future(_) => err!("unexpected form: future"),
//             #[cfg(feature = "async")]
//             Exp::Task(future) => future.get(),
//         }
//     }

//     pub fn apply(&self, exp: Exp, args: &[Exp], env: &SharedEnv) -> Result<Exp> {
//         match exp {
//             Exp::Function(f) => f(&self.eval_form(args, env)?, env, self),
//             Exp::Lambda(lambda) => self.apply_lambda(lambda, args, env),
//             Exp::Macro(lambda) => self.apply_macro(lambda, args, env),
//             _ => err!("first form must be function, lambda or macro"),
//         }
//     }

//     pub fn eval_form(&self, args: &[Exp], env: &SharedEnv) -> Result<Vec<Exp>> {
//         args.iter().map(|x| self.eval(x, env)).collect()
//     }

//     fn apply_lambda(&self, lambda: LambdaExp, args: &[Exp], env: &SharedEnv) -> Result<Exp> {
//         let keys = parse_list_of_symbol_strings(&lambda.params_exp)?;
//         if keys.len() != args.len() {
//             err!(format!(
//                 "expected {} arguments, got {}",
//                 keys.len(),
//                 args.len()
//             ))
//         }
//         let values = self.eval_form(args, env)?;
//         let child = Env::extend(env.clone(), &keys, &values);
//         self.eval(&lambda.body_exp, &child)
//     }

//     fn apply_macro(&self, lambda: LambdaExp, args: &[Exp], env: &SharedEnv) -> Result<Exp> {
//         let expanded = self.expand_macro(lambda, args, env)?;
//         self.eval(&expanded, &env)
//     }

//     pub fn expand_macro(&self, lambda: LambdaExp, args: &[Exp], env: &SharedEnv) -> Result<Exp> {
//         let keys = parse_list_of_symbol_strings(&lambda.params_exp)?;
//         if keys.len() != args.len() {
//             err!(format!(
//                 "expected {} arguments, got {}",
//                 keys.len(),
//                 args.len()
//             ))
//         }
//         let child = Env::extend(env.clone(), &keys, &args);
//         self.eval_quasiquote(&lambda.body_exp, &child)
//     }

//     // pub fn eval_quasiquote(&self, exp: &Exp, env: &SharedEnv) -> Result<Exp> {
//     //     let Exp::List(list) = exp else {
//     //         return Ok(exp.clone());
//     //     };
//     //     let mut result = Vec::new();
//     //     for item in list {
//     //         match item {
//     //             Exp::Symbol(s) if s == "quasiquote" => continue,
//     //             Exp::List(inner) if !inner.is_empty() => match &inner[0] {
//     //                 Exp::Symbol(s) if s == "unquote" => {
//     //                     if inner.len() != 2 {
//     //                         err!("unquote expects one argument")
//     //                     }
//     //                     let evaluated = self.eval(&inner[1], env)?;
//     //                     result.push(evaluated);
//     //                 }
//     //                 Exp::Symbol(s) if s == "unquote-splicing" => {
//     //                     if inner.len() != 2 {
//     //                         err!("unquote-splicing expects one argument")
//     //                     }
//     //                     let evaluated = self.eval(&inner[1], env)?;
//     //                     if let Exp::List(splice_items) = evaluated {
//     //                         result.extend(splice_items);
//     //                     } else {
//     //                         err!("unquote-splicing requires list result")
//     //                     }
//     //                 }
//     //                 _ => {
//     //                     let quoted = self.eval_quasiquote(item, env)?;
//     //                     result.push(quoted);
//     //                 }
//     //             },
//     //             _ => {
//     //                 let quoted = self.eval_quasiquote(item, env)?;
//     //                 result.push(quoted);
//     //             }
//     //         }
//     //     }

//     //     ok!(result)
//     // }
//     pub fn eval_quasiquote(&self, exp: &Exp, env: &SharedEnv) -> Result<Exp> {
//         match exp {
//             Exp::List(list) if !list.is_empty() => {
//                 match &list[0] {
//                     Exp::Symbol(s) if s == "quasiquote" => {
//                         if list.len() != 2 {
//                             err!("quasiquote expects one argument")
//                         }
//                         self.eval_quasiquote(&list[1], env)
//                     }
//                     Exp::Symbol(s) if s == "unquote" => {
//                         // unquote のみ評価
//                         self.eval(&list[1], env)
//                     }
//                     Exp::Symbol(s) if s == "unquote-splicing" => {
//                         // splicing も同様
//                         let val = self.eval(&list[1], env)?;
//                         if let Exp::List(items) = val {
//                             Ok(Exp::List(items)) // ここではまだリスト化
//                         } else {
//                             err!("unquote-splicing requires list")
//                         }
//                     }
//                     _ => {
//                         // 再帰展開
//                         let mut new_list = Vec::new();
//                         for item in list {
//                             new_list.push(self.eval_quasiquote(item, env)?);
//                         }
//                         Ok(Exp::List(new_list))
//                     }
//                 }
//             }
//             _ => Ok(exp.clone()), // シンボルや数値はそのまま
//         }
//     }
// }

// pub fn parse_list_of_symbol_strings(form: &Shared<Exp>) -> Result<Vec<String>> {
//     match form.as_ref() {
//         Exp::List(list) => list
//             .iter()
//             .map(|x| match x {
//                 Exp::Symbol(s) => Ok(s.clone()),
//                 _ => err!("expected symbol in the argument list"),
//             })
//             .collect(),
//         _ => err!("expected args form to be a list"),
//     }
// }
