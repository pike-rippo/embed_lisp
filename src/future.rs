#[cfg(feature = "async")]
use tokio::task::JoinHandle;

#[cfg(feature = "async")]
use crate::{
    GLOBAL_RUNTIME, evaluator::Evaluator, expression::Exp, flow::EvalResult, typedef::SharedEnv,
};

#[cfg(feature = "async")]
pub struct FutureExp {
    exp: Box<Exp>,
    env: SharedEnv,
    eval: Evaluator,
}

#[cfg(feature = "async")]
impl Clone for FutureExp {
    fn clone(&self) -> Self {
        Self {
            exp: self.exp.clone(),
            env: self.env.deep_copy(),
            eval: self.eval.deep_copy(),
        }
    }
}

#[cfg(feature = "async")]
impl FutureExp {
    pub fn new(exp: &Exp, env: &SharedEnv, eval: &Evaluator) -> Self {
        Self {
            exp: Box::new(exp.clone()),
            env: env.deep_copy(),
            eval: eval.deep_copy(),
        }
    }

    pub fn spawn(&self) -> JoinHandle<EvalResult> {
        let copy_exp = self.exp.clone();
        let copy_env = self.env.deep_copy();
        let copy_eval = self.eval.deep_copy();
        // GLOBAL_RUNTIME.spawn(async move { copy_eval.eval(copy_exp, &copy_env) })
        GLOBAL_RUNTIME.spawn(async move { copy_eval.eval(&copy_exp, &copy_env) })
    }
}
