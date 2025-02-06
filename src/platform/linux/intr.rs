use std::{
    ffi::c_void,
    ptr::{self, null, null_mut},
    sync::Mutex,
};

use libc::SIG_BLOCK;

use crate::{
    error::{UtcpErr, UtcpResult},
    platform::IRQEntry,
};

static SIGMASK: Mutex<libc::sigset_t> = Mutex::new(unsafe { std::mem::zeroed() });
static BARRIER: Mutex<libc::pthread_barrier_t> = Mutex::new(unsafe { std::mem::zeroed() });
static IRQS: Mutex<Vec<IRQEntry>> = Mutex::new(vec![]);

pub struct IntrContext {
    tid: libc::pthread_t,
}

impl IntrContext {
    pub fn new() -> Self {
        // intr_init
        let tid = unsafe { libc::pthread_self() };
        {
            let mut barrier = BARRIER.lock().unwrap();
            unsafe {
                libc::pthread_barrier_init(&mut *barrier, null(), 2);
            };
        }
        {
            let mut sigmask = SIGMASK.lock().unwrap();
            unsafe {
                libc::sigemptyset(&mut *sigmask);
                // notify the intr thread exit
                libc::sigaddset(&mut *sigmask, libc::SIGHUP);
            }
        }
        IntrContext { tid }
    }

    pub fn run(&mut self) -> UtcpResult<()> {
        let sigmask = SIGMASK.lock().unwrap();
        let err = unsafe { libc::pthread_sigmask(SIG_BLOCK, &*sigmask, ptr::null_mut()) };
        if err != 0 {
            return Err(UtcpErr::Intr(format!("pthread_sigmask failed: {}", err)));
        }
        let err =
            unsafe { libc::pthread_create(&mut self.tid, null(), intr_thread, ptr::null_mut()) };
        if err != 0 {
            return Err(UtcpErr::Intr(format!("pthread_create failed: {}", err)));
        }
        let mut barrier = BARRIER.lock().unwrap();
        unsafe { libc::pthread_barrier_wait(&mut *barrier) };

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

    let mut barrier = BARRIER.lock().unwrap();
    let _ = unsafe { libc::pthread_barrier_wait(&mut *barrier) };

    let mut terminate = false;
    while !terminate {
        {
            let sigmask = SIGMASK.lock().unwrap();
            let mut sig = 0;
            let err = unsafe { libc::sigwait(&*sigmask, &mut sig) };
            if err != 0 {
                log::error!("sigwait failed: {}", err);
                break;
            }
            if sig == libc::SIGHUP {
                terminate = true;
            } else {
                let irqs = IRQS.lock().unwrap();
                for irq in &*irqs {
                    if irq.irq == sig as u32 {
                        // TODO:
                        //irq.handler(irq.data);
                    }
                }
            }
        }
    }
    
    log::debug!("intr thread terminated");
    ptr::null_mut()
}
