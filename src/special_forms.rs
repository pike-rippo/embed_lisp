use crate::evaluator::Evaluator;

mod binding;
mod control;
mod core;
mod logic;
mod macros;
mod native;

pub use core::begin_impl;

pub fn register_all_special_form(eval: &Evaluator) {
    binding::register(eval);
    control::register(eval);
    core::register(eval);
    macros::register(eval);
    native::register(eval);
}
