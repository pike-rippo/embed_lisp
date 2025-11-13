use std::{
    fmt::Display,
    fs::File,
    io::{Read, Write},
};

use parking_lot::RwLock;

use crate::{
    Evaluator,
    error::{Error, Result, SyntaxError},
    exp::{Callable, Exp},
    flow::EvalResult,
    typedef::Shared,
};

enum FileInner {
    Stdin(std::io::Stdin),
    Stdout(std::io::Stdout),
    Stderr(std::io::Stderr),
    Regular(File, String),
}

impl Display for FileInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stdin(_) => write!(f, "stdin"),
            Self::Stdout(_) => write!(f, "stdout"),
            Self::Stderr(_) => write!(f, "stderr"),
            FileInner::Regular(_, path) => write!(f, "File {}", path),
        }
    }
}

impl FileInner {
    pub fn stdin() -> Self {
        Self::Stdin(std::io::stdin())
    }

    pub fn stdout() -> Self {
        Self::Stdout(std::io::stdout())
    }

    pub fn stderr() -> Self {
        Self::Stderr(std::io::stderr())
    }

    pub fn regular(file: File, path: String) -> Self {
        Self::Regular(file, path)
    }
}

pub struct FileObject {
    inner: RwLock<Option<FileInner>>,
}

pub fn register(eval: &Evaluator) {
    eval.register_native_object_creator("File", |args: &[Exp]| {
        if args.is_empty() || args.len() > 2 {
            return Err(SyntaxError::too_many_args("File", 2, args.len()));
        }
        let path = match &args[0] {
            Exp::Number(n) => match n {
                0.0 => return Ok(Exp::Native(Shared::new(FileObject::stdin())).value_flow()),
                1.0 => return Ok(Exp::Native(Shared::new(FileObject::stdout())).value_flow()),
                2.0 => return Ok(Exp::Native(Shared::new(FileObject::stderr())).value_flow()),
                _ => return Err(SyntaxError::invalid_args_type_nth("File", "string", 1)),
            },
            Exp::String(s) if s.eq_ignore_ascii_case("stdin") => {
                return Ok(Exp::Native(Shared::new(FileObject::stdin())).value_flow());
            }
            Exp::String(s) if s.eq_ignore_ascii_case("stdout") => {
                return Ok(Exp::Native(Shared::new(FileObject::stdout())).value_flow());
            }
            Exp::String(s) if s.eq_ignore_ascii_case("stderr") => {
                return Ok(Exp::Native(Shared::new(FileObject::stderr())).value_flow());
            }
            Exp::String(path) => path,
            _ => return Err(SyntaxError::invalid_args_type_nth("File", "string", 1)),
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
                _ => return Err(SyntaxError::Reason(format!("invalid file mode: {}", mode)).into()),
            }
        } else {
            options.read(true).write(true).create(true).truncate(true);
        }

        let file = options
            .open(path)
            .map_err(|e| Error::Reason(format!("failed to open {}: {}", path, e)))?;
        Ok(Exp::Native(Shared::new(FileObject::regular(file, path.clone()))).value_flow())
    });
}

impl FileObject {
    fn new(inner: FileInner) -> Self {
        Self {
            inner: RwLock::new(Some(inner)),
        }
    }

    pub fn stdin() -> Self {
        Self::new(FileInner::stdin())
    }

    pub fn stdout() -> Self {
        Self::new(FileInner::stdout())
    }

    pub fn stderr() -> Self {
        Self::new(FileInner::stderr())
    }

    fn regular(file: File, path: String) -> Self {
        Self::new(FileInner::regular(file, path))
    }

    fn handle_close(&self, _args: &[Exp]) -> EvalResult {
        self.inner.write().take();
        Ok(Exp::Bool(true).value_flow())
    }

    fn handle_read(&self, _args: &[Exp]) -> EvalResult {
        let mut buf = String::new();
        let mut inner = self.inner.write();
        let _ = match inner
            .as_mut()
            .ok_or_else(|| Error::reason("file already closed"))?
        {
            FileInner::Regular(f, _) => f.read_to_string(&mut buf),
            FileInner::Stdin(stdin) => stdin.read_to_string(&mut buf),
            other => return Err(Error::Reason(format!("{} is not readable", other))),
        };
        if buf.is_empty() {
            Ok(Exp::Nil.value_flow())
        } else {
            Ok(Exp::String(buf).value_flow())
        }
    }

    fn handle_read_line(&self, _args: &[Exp]) -> EvalResult {
        let mut buf = String::new();
        let mut inner = self.inner.write();
        let _ = match inner
            .as_mut()
            .ok_or_else(|| Error::reason("file already closed"))?
        {
            FileInner::Stdin(stdin) => stdin.read_line(&mut buf),
            other => return Err(Error::Reason(format!("{} is cannot call read-line", other))),
        };
        if buf.is_empty() {
            Ok(Exp::Nil.value_flow())
        } else {
            Ok(Exp::String(buf.trim_end().to_string()).value_flow())
        }
    }

    fn handle_read_lines(&self, _args: &[Exp]) -> EvalResult {
        let mut buf = String::new();
        let mut inner = self.inner.write();
        let _ = match inner
            .as_mut()
            .ok_or_else(|| Error::reason("file already closed"))?
        {
            FileInner::Regular(f, _) => f.read_to_string(&mut buf),
            FileInner::Stdin(stdin) => stdin.read_to_string(&mut buf),
            other => return Err(Error::Reason(format!("{} is not readable", other))),
        };

        Ok(Exp::List(
            buf.lines()
                .map(|l| Exp::String(l.to_string()))
                .collect::<Vec<Exp>>(),
        )
        .value_flow())
    }

    fn handle_write(&self, args: &[Exp]) -> EvalResult {
        let mut inner = self.inner.write();
        match inner
            .as_mut()
            .ok_or_else(|| Error::reason("file already closed"))?
        {
            FileInner::Regular(f, _) => {
                args.iter()
                    .map(unwrap_for_write)
                    .try_for_each(|maybe_bytes| write_maybe_bytes(f, maybe_bytes))?;
            }
            FileInner::Stdout(stdout) => {
                args.iter()
                    .map(unwrap_for_write)
                    .try_for_each(|maybe_bytes| write_maybe_bytes(stdout, maybe_bytes))?;
            }
            FileInner::Stderr(stderr) => {
                args.iter()
                    .map(unwrap_for_write)
                    .try_for_each(|maybe_bytes| write_maybe_bytes(stderr, maybe_bytes))?;
            }
            other => return Err(Error::Reason(format!("{} is not writable", other))),
        };
        Ok(Exp::Bool(true).value_flow())
    }

    fn handle_writeln(&self, args: &[Exp]) -> EvalResult {
        let mut inner = self.inner.write();
        match inner
            .as_mut()
            .ok_or_else(|| Error::reason("file already closed"))?
        {
            FileInner::Regular(f, _) => {
                args.iter()
                    .map(unwrap_for_writeln)
                    .try_for_each(|maybe_bytes| write_maybe_string(f, maybe_bytes))?;
            }
            FileInner::Stdout(stdout) => {
                args.iter()
                    .map(unwrap_for_writeln)
                    .try_for_each(|maybe_bytes| write_maybe_string(stdout, maybe_bytes))?;
            }
            FileInner::Stderr(stderr) => {
                args.iter()
                    .map(unwrap_for_writeln)
                    .try_for_each(|maybe_bytes| write_maybe_string(stderr, maybe_bytes))?;
            }
            other => return Err(Error::Reason(format!("{} is not writable", other))),
        };
        Ok(Exp::Bool(true).value_flow())
    }

    fn handle_flush(&self, args: &[Exp]) -> EvalResult {
        if !args.is_empty() {
            return Err(SyntaxError::invalid_args_size("flush", 0, args.len()));
        }

        let mut inner = self.inner.write();
        let _ = match inner
            .as_mut()
            .ok_or_else(|| Error::reason("file already closed"))?
        {
            FileInner::Stdout(stdout) => stdout.flush(),
            FileInner::Stderr(stderr) => stderr.flush(),
            other => return Err(Error::Reason(format!("{} is not flushable", other))),
        };
        Ok(Exp::Bool(true).value_flow())
    }
}

impl Display for FileObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.inner.read();
        let inner_ref = inner.as_ref();
        match inner_ref {
            None => write!(f, "Closed File"),
            Some(FileInner::Stdin(_)) => write!(f, "Stdin"),
            Some(FileInner::Stdout(_)) => write!(f, "Stdout"),
            Some(FileInner::Stderr(_)) => write!(f, "Stderr"),
            Some(FileInner::Regular(_, path)) => write!(f, "File path={}", path),
        }
    }
}

impl Callable for FileObject {
    fn call_method(&self, method_name: &str, args: &[Exp]) -> EvalResult {
        match method_name {
            "close" => self.handle_close(args),
            "read" => self.handle_read(args),
            "read-line" => self.handle_read_line(args),
            "read-lines" => self.handle_read_lines(args),
            "write" => self.handle_write(args),
            "writeln" => self.handle_writeln(args),
            "flush" => self.handle_flush(args),
            _ => Err(SyntaxError::no_such_method("File", method_name)),
        }
    }
}

fn unwrap_for_write(e: &Exp) -> Result<&[u8]> {
    match e {
        Exp::String(s) => Ok(s.as_bytes()),
        _ => Err(SyntaxError::invalid_args_type("write", "string")),
    }
}

fn unwrap_for_writeln(e: &Exp) -> Result<String> {
    match e {
        Exp::String(s) => Ok(format!("{}\n", s)),
        _ => Err(SyntaxError::invalid_args_type("writeln", "string")),
    }
}

fn write_maybe_bytes(
    writable: &mut impl Write,
    maybe_bytes: std::result::Result<&[u8], Error>,
) -> Result<()> {
    let bytes = maybe_bytes?;
    writable
        .write_all(bytes)
        .or(Err(Error::reason("file write error")))?;
    Ok(())
}

fn write_maybe_string(
    writable: &mut impl Write,
    maybe_string: std::result::Result<String, Error>,
) -> Result<()> {
    let string = maybe_string?;
    writable
        .write_all(string.as_bytes())
        .or(Err(Error::reason("file write error")))?;
    Ok(())
}
