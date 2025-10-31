use std::{
    fs::File,
    io::{Read, Write},
};

use parking_lot::RwLock;

use crate::{
    Evaluator,
    error::{Error, SyntaxError},
    expression::Exp,
    flow::EvalResult,
    native::NativeObject,
    typedef::Shared,
};

pub struct FileObject {
    inner: RwLock<File>,
}

pub fn register(eval: &Evaluator) {
    eval.register_native_object_creator("File", |args: &[Exp]| {
        if args.is_empty() || args.len() > 2 {
            return Err(SyntaxError::too_many_args("File", 2, args.len()));
        }
        let Exp::String(path) = &args[0] else {
            // err!("open expects a string")
            return Err(SyntaxError::invalid_args_type_nth("File", "string", 1));
        };

        let mut options = std::fs::OpenOptions::new();
        if let Some(maybe_mode) = args.get(1) {
            let Exp::String(mode) = maybe_mode else {
                return Err(SyntaxError::invalid_args_type_nth("File", "string", 2));
            };
            match &mode[..] {
                "r" => {
                    options.read(true);
                }
                "r+" => {
                    options.read(true).write(true);
                }
                "w" => {
                    options.write(true).create(true).truncate(true);
                }
                "w+" => {
                    options.read(true).write(true).create(true).truncate(true);
                }
                "a" => {
                    options.write(true).create(true).append(true);
                }
                "a+" => {
                    options.read(true).write(true).create(true).append(true);
                }
                // _ => err!(format!("invalid file mode: {}", mode)),
                _ => return Err(SyntaxError::Reason(format!("invalid file mode: {}", mode)).into()),
            }
        } else {
            options.read(true).write(true).create(true).truncate(true);
        }

        let file = options
            .open(path)
            .map_err(|e| Error::Reason(format!("failed to open {}: {}", path, e)))?;
        Ok(Exp::Native(Shared::new(FileObject::new(file))).value_flow())
    });
}

impl FileObject {
    pub fn new(file: File) -> Self {
        Self {
            inner: RwLock::new(file),
        }
    }

    fn handle_read(&self, _args: &[Exp]) -> EvalResult {
        let mut buf = String::new();
        self.inner
            .write()
            .read_to_string(&mut buf)
            .or(Err(Error::reason("file read error")))?;
        Ok(Exp::String(buf).value_flow())
    }

    fn handle_write(&self, args: &[Exp]) -> EvalResult {
        args.iter()
            .map(|e| match e {
                Exp::String(s) => Ok(s.as_bytes()),
                _ => Err(SyntaxError::invalid_args_type("write", "string")),
            })
            .try_for_each(|maybe_bytes| {
                let bytes = maybe_bytes?;
                self.inner
                    .write()
                    .write_all(bytes)
                    .or(Err(Error::reason("file write error")))
            })?;
        Ok(Exp::Bool(true).value_flow())
    }

    fn handle_writeln(&self, args: &[Exp]) -> EvalResult {
        args.iter()
            .map(|e| match e {
                Exp::String(s) => Ok(format!("{}\n", s)),
                _ => Err(SyntaxError::invalid_args_type("writeln", "string")),
            })
            .try_for_each(|maybe_string| {
                let string = maybe_string?;
                self.inner
                    .write()
                    .write_all(string.as_bytes())
                    .or(Err(Error::reason("file write error")))
            })?;
        Ok(Exp::Bool(true).value_flow())
        // if args.len() != 1 {
        //     err!("write expects one argument")
        // }

        // let Exp::String(s) = &args[0] else {
        //     err!("write expects a string")
        // };

        // self.inner
        //     .write()
        //     .write_all(format!("{}\n", s).as_bytes())
        //     .or(Err(Error::from("file read error")))?;
        // Ok(Exp::Bool(true).value_flow())
    }
}

impl NativeObject for FileObject {
    fn get_type_name(&self) -> &'static str {
        "File"
    }

    fn call_method(&self, method_name: &str, args: &[Exp]) -> EvalResult {
        match method_name {
            "read" => self.handle_read(args),
            "write" => self.handle_write(args),
            "writeln" => self.handle_writeln(args),
            // _ => Err(Error::Reason(format!("unknown method '{}'", method_name))),
            _ => Err(SyntaxError::no_such_method("File", method_name)),
        }
    }
}
