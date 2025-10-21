use std::{
    cell::RefCell,
    fs::File,
    io::{Read, Write},
};

use crate::{
    err,
    error::{Error, Result},
    expression::Exp,
    native::NativeObject,
    ok,
};

pub struct FileObject {
    inner: RefCell<File>,
}

impl FileObject {
    pub fn new(file: File) -> Self {
        Self {
            inner: RefCell::new(file),
        }
    }
}

impl NativeObject for FileObject {
    fn get_type_name(&self) -> &'static str {
        "File"
    }
    fn call_method(&self, name: &str, args: &[Exp]) -> Result<Exp> {
        match name {
            "read" => {
                let mut buf = String::new();
                self.inner
                    .borrow_mut()
                    .read_to_string(&mut buf)
                    .or(Err(Error::from("file read error")))?;
                Ok(Exp::String(buf))
            }
            "write" => {
                if args.len() != 1 {
                    err!("write expects one argument")
                }
                let Exp::String(s) = &args[0] else {
                    err!("write expects a string")
                };
                self.inner
                    .borrow_mut()
                    .write_all(s.as_bytes())
                    .or(Err(Error::from("file read error")))?;
                ok!(true)
            }

            _ => err!(format!("unknown method '{}'", name)),
        }
    }
}
