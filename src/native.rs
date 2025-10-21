use crate::{error::Result, expression::Exp};

mod file;

pub trait NativeObject {
    fn get_type_name(&self) -> &'static str;
    fn call_method(&self, name: &str, args: &[Exp]) -> Result<Exp>;
}

pub use file::FileObject;
