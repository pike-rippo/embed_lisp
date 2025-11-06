use crate::environment::SharedEnv;

mod compare;
mod core;
mod debug;
mod io;
mod list;
mod math;
mod namespace;
mod string;

pub fn register_all(env: &SharedEnv) {
    compare::register(env);
    core::register(env);
    debug::register(env);
    io::register(env);
    list::register(env);
    math::register(env);
    namespace::register(env);
    // string::register(env);
}
