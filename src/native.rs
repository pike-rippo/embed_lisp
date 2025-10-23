use crate::{error::Result, expression::Exp};

mod file;
mod hash_map;

pub use file::FileObject;
pub use hash_map::HashMapObject;

pub trait NativeObject {
    fn get_type_name(&self) -> &'static str;
    fn call_method(&self, method_name: &str, args: &[Exp]) -> Result<Exp>;
}
