use crate::typedef::SharedEnv;

mod file;
mod hash_map;

pub fn register(env: &SharedEnv) {
    file::register(env);
    hash_map::register(env);
}
