use std::{
    ffi::c_void,
    ptr::{self, null, null_mut},
};

use libc::SIG_BLOCK;

use crate::error::{UtcpErr, UtcpResult};

pub struct IntrContext {
    sigmask: libc::sigset_t,
    tid: libc::pthread_t,
    barrier: *mut libc::pthread_barrier_t,
}

impl IntrContext {
    pub fn new() -> Self {
        // intr_init
        let tid = unsafe { libc::pthread_self() };
        let barrier = unsafe { std::mem::zeroed() };
        unsafe {
            libc::pthread_barrier_init(barrier, null(), 2);
        };
        let mut sigmask = unsafe { std::mem::zeroed() };
        unsafe {
            libc::sigemptyset(&mut sigmask);
            // notify the intr thread exit
            libc::sigaddset(&mut sigmask, libc::SIGHUP);
        }
        IntrContext {
            sigmask,
            tid,
            barrier,
        }
    }

    pub fn run(&mut self) -> UtcpResult<()> {
        let err = unsafe { libc::pthread_sigmask(SIG_BLOCK, &self.sigmask, ptr::null_mut()) };
        if err != 0 {
            return Err(UtcpErr::Intr(format!("pthread_sigmask failed: {}", err)));
        }
        let err =
            unsafe { libc::pthread_create(&mut self.tid, null(), intr_thread, ptr::null_mut()) };
        if err != 0 {
            return Err(UtcpErr::Intr(format!("pthread_create failed: {}", err)));
        }
        unsafe { libc::pthread_barrier_wait(self.barrier) };

        Ok(())
    }

    fn shutdown(&mut self) {
        unsafe {
            let tid = libc::pthread_self();
            if libc::pthread_equal(tid, self.tid) != 0 {
                // thread not created
                return;
            }
            // Send SIGHUP to notify the intr thread
            libc::pthread_kill(self.tid, libc::SIGHUP);
            // Wait for the intr thread to exit
            libc::pthread_join(self.tid, null_mut());
        }
    }
}

impl Drop for IntrContext {
    fn drop(&mut self) {
        self.shutdown();
    }
}

extern "C" fn intr_thread(_: *mut c_void) -> *mut c_void {
    log::debug!("intr thread start");

    // TODO:
    ptr::null_mut()
}
