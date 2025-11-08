#[cfg(feature = "async")]
use tokio::sync::Mutex;

#[cfg(feature = "async")]
use tokio::task::JoinHandle;

#[cfg(feature = "async")]
use crate::{error::Error, flow::EvalResult, typedef::Shared};

#[cfg(feature = "async")]
#[derive(Clone)]
pub struct TaskExp {
    handle: Shared<Mutex<Option<JoinHandle<EvalResult>>>>,
}

#[cfg(feature = "async")]
impl PartialEq for TaskExp {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&*self.handle, &*other.handle)
    }
}

#[cfg(feature = "async")]
impl TaskExp {
    pub fn new(handle: JoinHandle<EvalResult>) -> Self {
        Self {
            handle: Shared::new(Mutex::new(Some(handle))),
        }
    }

    pub fn status(&self) -> &'static str {
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::block_in_place(|| {
                let Some(handle) = self.handle.blocking_lock().take() else {
                    return "Consumed";
                };
                let r = handle.is_finished();
                self.handle.blocking_lock().replace(handle);
                if r { "Ready" } else { "Pending" }
            })
        } else {
            let Some(handle) = self.handle.blocking_lock().take() else {
                return "Consumed";
            };
            let r = handle.is_finished();
            self.handle.blocking_lock().replace(handle);
            if r { "Ready" } else { "Pending" }
        }
    }

    pub fn is_ready(&self) -> bool {
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::block_in_place(|| {
                let Some(handle) = self.handle.blocking_lock().take() else {
                    return false;
                };
                let r = handle.is_finished();
                self.handle.blocking_lock().replace(handle);
                r
            })
        } else {
            let Some(handle) = self.handle.blocking_lock().take() else {
                return false;
            };
            let r = handle.is_finished();
            self.handle.blocking_lock().replace(handle);
            r
        }
    }

    pub fn get(&self) -> EvalResult {
        Err(Error::reason("Pending"))
    }

    pub fn sync_await(&self) -> EvalResult {
        use crate::GLOBAL_RUNTIME;

        let Some(handle) = self.handle.blocking_lock().take() else {
            return Err(Error::reason("no handle"));
        };

        let result = GLOBAL_RUNTIME.block_on(handle);

        match result {
            Ok(r) => r,
            Err(e) => {
                println!("{}", e);
                Err(Error::reason("join error"))
            }
        }
    }

    pub async fn async_await(&self) -> EvalResult {
        self.handle
            .lock()
            .await
            .take()
            .ok_or_else(|| Error::reason("no handle"))?
            .await
            .map_err(|_| Error::reason("join error"))?
    }
}
