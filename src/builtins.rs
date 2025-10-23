use crate::typedef::SharedEnv;

mod compare;
mod core;
mod debug;
mod io;
mod list;
mod math;
mod native;

pub fn register_all(env: &SharedEnv) {
    compare::register(env);
    core::register(env);
    debug::register(env);
    io::register(env);
    math::register(env);
    list::register(env);
    native::register(env);
}
