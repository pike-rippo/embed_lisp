use crate::{
    expression::Exp,
    native::HashMapObject,
    typedef::{Shared, SharedEnv},
};

pub fn register(env: &SharedEnv) {
    env.define(
        "create-hash-map",
        Exp::Function(|_, _, _| Ok(Exp::Native(Shared::new(HashMapObject::new())))),
    );
}
