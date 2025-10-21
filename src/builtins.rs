use crate::environment::EnvRc;

mod compare;
mod core;
mod debug;
mod io;
mod list;
mod math;
mod native;

pub fn register_all(env: &EnvRc) {
    compare::register(env);
    core::register(env);
    debug::register(env);
    io::register(env);
    math::register(env);
    list::register(env);
    native::register(env);
}
