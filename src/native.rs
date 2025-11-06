use crate::Evaluator;

mod file;
mod hash_map;
mod hash_set;

pub use file::FileObject;
pub use hash_map::HashMapObject;
pub use hash_set::HashSetObject;

pub fn register_all_native_object_creator(eval: &Evaluator) {
    file::register(eval);
    hash_map::register(eval);
    hash_set::register(eval);
}
