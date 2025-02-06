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

static TID: Mutex<libc::pthread_t> = Mutex::new(libc::pthread_t::MAX);
static SIGMASK: Mutex<libc::sigset_t> = Mutex::new(unsafe { std::mem::zeroed() });
static mut BARRIER: libc::pthread_barrier_t = unsafe { std::mem::zeroed() };
static IRQS: Mutex<Vec<IRQEntry>> = Mutex::new(vec![]);

pub fn intr_init() -> UtcpResult<()> {
    log::debug!("intr init");
    {
        let tid = unsafe { libc::pthread_self() };
        let mut tid_lock = TID.lock().unwrap();
        *tid_lock = tid;
    }
    {
        unsafe {
            libc::pthread_barrier_init(&raw mut BARRIER, null(), 2);
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
    log::debug!("intr init");
    Ok(())
}

pub fn intr_run() -> UtcpResult<()> {
    let sigmask = SIGMASK.lock().unwrap();
    let err = unsafe { libc::pthread_sigmask(SIG_BLOCK, &*sigmask, ptr::null_mut()) };
    if err != 0 {
        return Err(UtcpErr::Intr(format!("pthread_sigmask failed: {}", err)));
    }
    let mut tid = TID.lock().unwrap();
    let err = unsafe { libc::pthread_create(&mut *tid, null(), intr_thread, ptr::null_mut()) };
    if err != 0 {
        return Err(UtcpErr::Intr(format!("pthread_create failed: {}", err)));
    }
    unsafe { libc::pthread_barrier_wait(&raw mut BARRIER) };

    Ok(())
}

pub fn intr_shutdown() -> UtcpResult<()> {
    unsafe {
        let tid = TID.lock().unwrap();
        let current_tid = libc::pthread_self();
        if libc::pthread_equal(current_tid, *tid) != 0 {
            // thread not created
            return Ok(());
        }
        // Send SIGHUP to notify the intr thread
        libc::pthread_kill(*tid, libc::SIGHUP);
        // Wait for the intr thread to exit
        libc::pthread_join(*tid, null_mut());
    }
    Ok(())
}

extern "C" fn intr_thread(_: *mut c_void) -> *mut c_void {
    log::debug!("intr thread start");

    let _ = unsafe { libc::pthread_barrier_wait(&raw mut BARRIER) };

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
