#[cfg(not(feature = "async"))]
use std::rc::Rc;
#[cfg(feature = "async")]
use std::sync::Arc;

#[cfg(not(feature = "async"))]
pub type Shared<T> = Rc<T>;
#[cfg(feature = "async")]
pub type Shared<T> = Arc<T>;
