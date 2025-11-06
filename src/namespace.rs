use crate::environment::SharedEnv;

mod string;

pub fn register_all(env: &SharedEnv) {
    string::register(env);
}
