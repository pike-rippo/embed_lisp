use crate::{Evaluator, expression::Exp, flow::EvalResult};

mod file;
mod hash_map;
mod hash_set;

pub trait NativeObject {
    fn get_type_name(&self) -> &'static str;
    fn call_method(&self, name: &str, args: &[Exp]) -> EvalResult;
}

pub fn register_all_native_object_creator(eval: &Evaluator) {
    file::register(eval);
    hash_map::register(eval);
    hash_set::register(eval);
}
