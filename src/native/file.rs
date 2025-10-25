use std::{
    fs::File,
    io::{Read, Write},
};

use parking_lot::RwLock;

use crate::{
    Evaluator, err,
    error::{Error, Result},
    expression::Exp,
    native::NativeObject,
    ok,
    typedef::Shared,
};

pub struct FileObject {
    inner: RwLock<File>,
}

pub fn register(eval: &Evaluator) {
    eval.register_native_object_creator("File", |args: &[Exp]| {
        if args.is_empty() {
            err!("open expects one argument")
        }
        let Exp::String(path) = &args[0] else {
            err!("open expects a string")
        };

        let mut options = std::fs::OpenOptions::new();
        if let Some(maybe_mode) = args.get(1) {
            let Exp::String(mode) = maybe_mode else {
                err!("second argument must be a string")
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
                _ => err!(format!("invalid file mode: {}", mode)),
            }
        } else {
            options.read(true).write(true).create(true).truncate(true);
        }

        let file = options
            .open(path)
            .map_err(|e| format!("failed to open {}: {}", path, e))?;
        Ok(Exp::Native(Shared::new(FileObject::new(file))))
    });
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
