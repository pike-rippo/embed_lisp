use crate::{expression::Exp, typedef::Shared};

#[derive(Debug, Clone, PartialEq)]
pub struct LambdaExp {
    pub params_exp: Shared<Exp>,
    pub body_exp: Shared<Exp>,
}

impl LambdaExp {
    pub fn new(params_exp: Shared<Exp>, body_exp: Shared<Exp>) -> Self {
        LambdaExp {
            params_exp,
            body_exp,
        }
    }
}
