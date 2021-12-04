use futures::future::Future;

pub struct AsyncRT {
    pub local_set: tokio::task::LocalSet,
    pub tokio_rt: tokio::runtime::Runtime
}

impl AsyncRT {
    pub fn new() -> Self {
        let tokio_rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();
        AsyncRT {
            tokio_rt,
            local_set: tokio::task::LocalSet::new()
        }
    }

    #[inline]
    pub fn block_on<F>(&self, future: F) -> F::Output
        where F: Future, {
            let fut = self.local_set.run_until(future);
            self.tokio_rt.block_on(fut)
    }
}

impl From<u32> for Box<AsyncRT> {
    fn from(ptr: u32) -> Box<AsyncRT> {
        let async_rt = ptr as *mut AsyncRT;
        unsafe {
            Box::from_raw(async_rt)
        }
    }
}

