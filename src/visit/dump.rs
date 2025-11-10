use crate::{
    Exp,
    environment::Env,
    exp::NameSpace,
    visit::{HashMapIter, Visitor},
};

use std::fmt::Write;

pub struct DumpVisitor {
    depth: usize,
    indent: &'static str,
    buf: String,
}

impl Default for DumpVisitor {
    fn default() -> Self {
        Self {
            depth: 0,
            indent: "   ",
            buf: String::with_capacity(1024),
        }
    }
}

impl DumpVisitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn accept_hash_map_iter(&mut self, indent: &str, hash_map_iter: HashMapIter) {
        for (k, v) in hash_map_iter {
            if v.is_namespace() {
                let _ = write!(self.buf, "{}{}", indent, k);
            } else {
                let _ = write!(self.buf, "{}'{}' = ", indent, k);
            }
            self.visit_exp(v);
            let _ = self.buf.write_char('\n');
        }
    }
}

impl Visitor for DumpVisitor {
    fn result(&self) -> &str {
        &self.buf
    }

    fn visit_env(&mut self, env: &Env) {
        let indent = self.indent.repeat(self.depth);
        let _ = match env.level() {
            0 | 1 => return,
            2 => writeln!(self.buf, "{}Global", indent),
            level => writeln!(self.buf, "{}Level: {}", indent, level - 2),
        };

        self.depth += 1;
        let indent = self.indent.repeat(self.depth);
        self.accept_hash_map_iter(&indent, env.current_iter());
        let _ = self.buf.write_char('\n');
        self.depth -= 1;

        env.parent_accept(self);
    }
    fn visit_exp(&mut self, exp: &Exp) {
        match exp {
            Exp::Namespace(ns) => self.visit_ns(ns),
            _ => {
                let _ = write!(self.buf, "{}", exp);
            }
        };
    }
    fn visit_ns(&mut self, ns: &NameSpace) {
        let _ = writeln!(self.buf, "::");
        self.depth += 1;
        let indent = self.indent.repeat(self.depth);
        self.accept_hash_map_iter(&indent, ns.current_iter());
        self.depth -= 1;
    }
}
