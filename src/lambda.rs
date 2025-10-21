use std::rc::Rc;

use crate::expression::Exp;

#[derive(Debug, Clone, PartialEq)]
pub struct LambdaExp {
    pub params_exp: Rc<Exp>,
    pub body_exp: Rc<Exp>,
}

impl LambdaExp {
    pub fn new(params_exp: Rc<Exp>, body_exp: Rc<Exp>) -> Self {
        LambdaExp {
            params_exp,
            body_exp,
        }
    }
}
