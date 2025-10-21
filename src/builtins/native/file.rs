use std::rc::Rc;

use crate::{
    environment::EnvRc, err, error::Result, evaluator::Evaluator, expression::Exp,
    native::FileObject,
};

pub fn register(env: &EnvRc) {
    env.define("open", Exp::Function(open_impl));
}

fn open_impl(args: &[Exp], _: &EnvRc, _: &Evaluator) -> Result<Exp> {
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
    Ok(Exp::Native(Rc::new(FileObject::new(file))))
}
