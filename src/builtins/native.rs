use crate::typedef::SharedEnv;

mod file;

pub fn register(env: &SharedEnv) {
    file::register(env);
}
