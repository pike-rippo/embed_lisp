use crate::Evaluator;

mod file;
mod hash_map;
mod hash_set;

pub fn register_all_native_object_creator(eval: &Evaluator) {
    file::register(eval);
    hash_map::register(eval);
    hash_set::register(eval);
}
