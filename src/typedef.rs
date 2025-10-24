#[cfg(not(feature = "async"))]
use std::rc::Rc;
#[cfg(feature = "async")]
use std::sync::Arc;

use crate::environment::Env;

#[cfg(not(feature = "async"))]
pub type SharedEnv = Rc<Env>;
#[cfg(feature = "async")]
pub type SharedEnv = Arc<Env>;

#[cfg(not(feature = "async"))]
pub type Shared<T> = Rc<T>;
#[cfg(feature = "async")]
pub type Shared<T> = Arc<T>;

// #[cfg(not(feature = "async"))]
// pub type SharedMut<T> = RefCell<T>;
// #[cfg(feature = "async")]
// pub type SharedMut<T> = Mutex<T>;
