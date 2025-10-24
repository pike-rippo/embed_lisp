use std::{
    fs::File,
    io::{Read, Write},
    sync::RwLock,
};

use crate::{
    err,
    error::{Error, Result},
    expression::Exp,
    native::NativeObject,
    ok, write_error,
};

pub struct FileObject {
    inner: RwLock<File>,
}

impl FileObject {
    pub fn new(file: File) -> Self {
        Self {
            inner: RwLock::new(file),
        }
    }

    fn handle_read(&self, _args: &[Exp]) -> Result<Exp> {
        let mut buf = String::new();
        self.inner
            .write()
            .or(write_error!())?
            .read_to_string(&mut buf)
            .or(Err(Error::from("file read error")))?;
        Ok(Exp::String(buf))
    }

    fn handle_write(&self, args: &[Exp]) -> Result<Exp> {
        if args.len() != 1 {
            err!("write expects one argument")
        }

        let Exp::String(s) = &args[0] else {
            err!("write expects a string")
        };

        self.inner
            .write()
            .or(write_error!())?
            .write_all(s.as_bytes())
            .or(Err(Error::from("file read error")))?;
        ok!(true)
    }

    fn handle_writeln(&self, args: &[Exp]) -> Result<Exp> {
        if args.len() != 1 {
            err!("write expects one argument")
        }

        let Exp::String(s) = &args[0] else {
            err!("write expects a string")
        };

        self.inner
            .write()
            .or(write_error!())?
            .write_all(format!("{}\n", s).as_bytes())
            .or(Err(Error::from("file read error")))?;
        ok!(true)
    }
}

impl NativeObject for FileObject {
    fn get_type_name(&self) -> &'static str {
        "File"
    }

    fn call_method(&self, method_name: &str, args: &[Exp]) -> Result<Exp> {
        match method_name {
            "read" => self.handle_read(args),
            "write" => self.handle_write(args),
            "writeln" => self.handle_writeln(args),
            _ => err!(format!("unknown method '{}'", method_name)),
        }
    }
}

// #[cfg(not(feature = "async"))]
// impl NativeObject for FileObject {
//     fn get_type_name(&self) -> &'static str {
//         "File"
//     }
//     fn call_method(&self, name: &str, args: &[Exp]) -> Result<Exp> {
//         match name {
//             "read" => {
//                 let mut buf = String::new();
//                 self.inner
//                     .borrow_mut()
//                     .read_to_string(&mut buf)
//                     .or(Err(Error::from("file read error")))?;
//                 Ok(Exp::String(buf))
//             }
//             "write" => {
//                 if args.len() != 1 {
//                     err!("write expects one argument")
//                 }
//                 let Exp::String(s) = &args[0] else {
//                     err!("write expects a string")
//                 };
//                 self.inner
//                     .borrow_mut()
//                     .write_all(s.as_bytes())
//                     .or(Err(Error::from("file read error")))?;
//                 ok!(true)
//             }

//             _ => err!(format!("unknown method '{}'", name)),
//         }
//     }
// }

// #[cfg(feature = "async")]
// impl NativeObject for FileObject {
//     fn get_type_name(&self) -> &'static str {
//         "File"
//     }
//     fn call_method(&self, name: &str, args: &[Exp]) -> Result<Exp> {
//         match name {
//             "read" => {
//                 let mut buf = String::new();
//                 if tokio::runtime::Handle::try_current().is_ok() {
//                     if tokio::task::block_in_place(|| {
//                         self.inner.blocking_lock().read_to_string(&mut buf)
//                     })
//                     .is_err()
//                     {
//                         return Err(Error::from("file read error"));
//                     }
//                 } else {
//                     self.inner
//                         .blocking_lock()
//                         .read_to_string(&mut buf)
//                         .or(Err(Error::from("file read error")))?;
//                 }
//                 Ok(Exp::String(buf))
//             }
//             "write" => {
//                 if args.len() != 1 {
//                     err!("write expects one argument")
//                 }
//                 let Exp::String(s) = &args[0] else {
//                     err!("write expects a string")
//                 };
//                 if tokio::runtime::Handle::try_current().is_ok() {
//                     if tokio::task::block_in_place(|| {
//                         self.inner.blocking_lock().write_all(s.as_bytes())
//                     })
//                     .is_err()
//                     {
//                         return Err(Error::from("file write error"));
//                     }
//                 } else {
//                     self.inner
//                         .blocking_lock()
//                         .write_all(s.as_bytes())
//                         .or(Err(Error::from("file write error")))?;
//                 }
//                 ok!(true)
//             }

//             _ => err!(format!("unknown method '{}'", name)),
//         }
//     }
// }
