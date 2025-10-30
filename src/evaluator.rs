use std::{
    collections::HashMap,
    ops::Deref,
    sync::atomic::{AtomicBool, AtomicUsize},
};

use parking_lot::RwLock;

use crate::{
    GENESYM_COUNTER,
    environment::Env,
    err,
    error::{Error, Result},
    expression::Exp,
    flow::{EvalFlow, EvalResult},
    lambda::LambdaExp,
    native,
    native_registry::{NativeCreator, NativeRegistry},
    special_forms::{self, begin_impl},
    typedef::{Shared, SharedEnv},
};

pub type SpecialFormFn = fn(&[Exp], &SharedEnv, &Evaluator) -> EvalResult;

pub struct Evaluator {
    special_forms: RwLock<HashMap<String, SpecialFormFn>>,
    native_registry: NativeRegistry,
    trace: AtomicBool,
    gensym_counter: &'static AtomicUsize,
}

#[cfg(feature = "async")]
impl Clone for Evaluator {
    fn clone(&self) -> Self {
        self.deep_copy()
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator {
    pub fn new() -> Self {
        let eval = Self {
            special_forms: RwLock::new(HashMap::new()),
            native_registry: NativeRegistry::new(),
            trace: AtomicBool::new(false),
            gensym_counter: &GENESYM_COUNTER,
        };

        special_forms::register_all_special_form(&eval);
        native::register_all_native_object_creator(&eval);

        eval
    }

    #[cfg(feature = "async")]
    pub fn deep_copy(&self) -> Self {
        Self {
            special_forms: RwLock::new(self.special_forms.read().clone()),
            native_registry: self.native_registry.clone(),
            trace: AtomicBool::new(self.trace.load(std::sync::atomic::Ordering::Relaxed)),
            gensym_counter: &GENESYM_COUNTER,
        }
    }

    pub fn register_special_form(&self, k: &str, f: SpecialFormFn) {
        self.special_forms.write().insert(k.to_string(), f);
    }

    pub fn register_native_object_creator(&self, k: &str, creator: NativeCreator) {
        self.native_registry.register(k, creator);
    }

    pub fn create_native_object(&self, name: &str, args: &[Exp]) -> EvalResult {
        self.native_registry.create(name, args)
    }

    pub fn native_object_creator_keys(&self) -> Vec<String> {
        self.native_registry.keys().to_vec()
    }

    pub fn get_gensym_id(&self) -> usize {
        self.gensym_counter
            .fetch_add(1, std::sync::atomic::Ordering::Release)
    }

    pub fn set_trace(&self, value: bool) {
        self.trace
            .store(value, std::sync::atomic::Ordering::Release);
    }

    pub fn eval(&self, exp: &Exp, env: &SharedEnv) -> EvalResult {
        if self.trace.load(std::sync::atomic::Ordering::Relaxed) {
            println!("trace eval: {}", exp);
        }

        match exp {
            Exp::Nil | Exp::Number(_) | Exp::Bool(_) | Exp::String(_) | Exp::Native(_) => {
                Ok(EvalFlow::Value(exp.clone()))
            }
            Exp::Function(_) => Err(Error::from("unexpected form: Function")),
            Exp::Lambda(_) => err!("unexpected form: lambda"),
            Exp::Macro(_) => err!("unexpected form: macro"),
            Exp::DottedList(_, _) => err!("unexpected form: dotted list"),
            Exp::Symbol(k) => Ok(env.try_lookup(k)?.value_flow()),
            Exp::List(list) => {
                let Some(first_form) = list.first() else {
                    return Ok(EvalFlow::Value(Exp::Nil));
                };
                let args = &list[1..];
                if let Exp::Symbol(k) = first_form
                    && let Some(f) = self.special_forms.read().get(k)
                {
                    return f(args, env, self);
                }

                let first_eval = self.eval(first_form, env)?;
                self.apply(first_eval.try_unwrap()?, args, env)
            }
            #[cfg(feature = "async")]
            Exp::Future(_) => err!("unexpected form: future"),
            #[cfg(feature = "async")]
            Exp::Task(future) => future.get(),
        }
    }

    pub fn apply(&self, exp: Exp, args: &[Exp], env: &SharedEnv) -> EvalResult {
        match exp {
            Exp::Function(f) => Ok(f(&self.eval_form(args, env)?, env, self)?),
            Exp::Lambda(lambda) => self.apply_lambda(lambda, args, env),
            Exp::Macro(lambda) => self.apply_macro(lambda, args, env),
            _ => err!("first form must be function, lambda or macro"),
        }
    }

    pub fn eval_form(&self, args: &[Exp], env: &SharedEnv) -> Result<Vec<Exp>> {
        Ok(args
            .iter()
            .map(|x| self.eval(x, env))
            .collect::<Result<Vec<EvalFlow>>>()?
            .iter()
            .map(EvalFlow::try_unwrap)
            .collect::<Result<Vec<Exp>>>()?)
    }

    fn apply_lambda(&self, lambda: LambdaExp, args: &[Exp], env: &SharedEnv) -> EvalResult {
        let values = self.eval_form(args, env)?;
        let child = self.env_extend(&lambda.params_exp, &values, env)?;
        // self.eval(&lambda.body_exp, &child)
        begin_impl(&lambda.body_exp, &child, self)
    }

    fn apply_macro(&self, lambda: LambdaExp, args: &[Exp], env: &SharedEnv) -> EvalResult {
        let expanded = self.expand_macro(lambda, args, env)?;
        self.eval(&expanded.try_unwrap()?, env)
        // begin_impl(&expanded.try_unwrap()?, env, self)
    }

    pub fn expand_macro(&self, lambda: LambdaExp, args: &[Exp], env: &SharedEnv) -> EvalResult {
        let child = self.env_extend(&lambda.params_exp, args, env)?;
        // self.eval(&lambda.body_exp, &child)
        begin_impl(&lambda.body_exp, &child, self)
    }

    pub fn env_extend(
        &self,
        params: &Shared<Exp>,
        args: &[Exp],
        env: &SharedEnv,
    ) -> Result<SharedEnv> {
        let is_dotted = params.is_dotted_list();
        let keys = parse_list_of_symbol_strings(params)?;
        if is_dotted {
            Ok(Env::extend_dotted(env.clone(), &keys, args))
        } else if keys.len() == args.len() {
            Ok(Env::extend(env.clone(), &keys, args))
        } else {
            err!(format!(
                "expected {} arguments, got {}",
                keys.len(),
                args.len()
            ))
        }
    }

    pub fn eval_quasiquote(&self, exp: &Exp, env: &SharedEnv) -> Result<(Exp, bool)> {
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
                        Ok((self.eval(&list[1], env)?.try_unwrap()?, false))
                    }
                    Exp::Symbol(s) if s == "unquote-splicing" => {
                        let val = self.eval(&list[1], env)?;
                        if let Exp::List(items) = val.try_unwrap()? {
                            Ok((Exp::List(items), true))
                        } else {
                            err!("unquote-splicing requires list")
                        }
                    }
                    _ => {
                        // 再帰展開
                        let mut new_list = Vec::new();
                        for item in list {
                            let (exp, splicing) = self.eval_quasiquote(item, env)?;
                            if splicing {
                                let Exp::List(items) = exp else {
                                    unreachable!();
                                };
                                new_list.extend(items);
                            } else {
                                new_list.push(exp);
                            }
                        }
                        Ok((Exp::List(new_list), false))
                    }
                }
            }
            _ => Ok((exp.clone(), false)),
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
        Exp::DottedList(args, res) => {
            let Exp::Symbol(s) = res.deref() else {
                err!("expected symbol in the argument list")
            };
            let mut list = args
                .iter()
                .map(|e| match e {
                    Exp::Symbol(s) => Ok(s.clone()),
                    _ => err!("expected symbol in the argument list"),
                })
                .collect::<Result<Vec<String>>>()?;

            list.push(s.clone());
            Ok(list)
        }
        _ => err!("expected args form to be a list"),
    }
}
