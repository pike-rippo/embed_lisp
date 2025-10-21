use crate::environment::EnvRc;

mod file;

pub fn register(env: &EnvRc) {
    file::register(env);
}
