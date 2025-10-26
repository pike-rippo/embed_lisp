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
impl TaskExp {
    pub fn new(handle: JoinHandle<EvalResult>) -> Self {
        Self {
            handle: Shared::new(Mutex::new(Some(handle))),
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
        Err(Error::from("Pending"))
    }

    pub fn sync_await(&self) -> EvalResult {
        use crate::{GLOBAL_RUNTIME, err};

        let Some(handle) = self.handle.blocking_lock().take() else {
            err!("no handle")
        };

        let result = GLOBAL_RUNTIME.block_on(handle);

        match result {
            Ok(r) => r,
            Err(e) => {
                println!("{}", e);
                err!("join error")
            }
        }
    }

    pub async fn async_await(&self) -> EvalResult {
        self.handle
            .lock()
            .await
            .take()
            .ok_or_else(|| Error::from("no handle"))?
            .await
            .map_err(|_| Error::from("join error"))?
    }
}
