use std::{
    ffi::c_void,
    ptr::{self, null, null_mut},
};

use libc::SIG_BLOCK;

use crate::error::{UtcpErr, UtcpResult};

pub struct IntrContext {
    set: libc::sigset_t,
    tid: libc::pthread_t,
}

impl IntrContext {
    pub fn new() -> Self {
        let mut set = unsafe { std::mem::zeroed() };
        unsafe {
            libc::sigemptyset(&mut set);
            // notify the intr thread exit
            libc::sigaddset(&mut set, libc::SIGHUP);
        }
        let tid = 0;
        IntrContext { set, tid }
    }

    pub fn run(&mut self) -> UtcpResult<()> {
        // sigmask
        let err = unsafe { libc::pthread_sigmask(SIG_BLOCK, &self.set, ptr::null_mut()) };
        if err != 0 {
            return Err(UtcpErr::Intr(format!("pthread_sigmask failed: {}", err)));
        }
        let err =
            unsafe { libc::pthread_create(&mut self.tid, null(), intr_thread, ptr::null_mut()) };
        if err != 0 {
            return Err(UtcpErr::Intr(format!("pthread_create failed: {}", err)));
        }

        Ok(())
    }
}

extern "C" fn intr_thread(_: *mut c_void) -> *mut c_void {
    ptr::null_mut()
}
