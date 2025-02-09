use std::{
    ffi::{c_int, c_void},
    ptr::{self, null, null_mut},
    sync::Mutex,
};

use libc::SIG_BLOCK;

use crate::{
    driver::INTR_IRQ_SOFTIRQ,
    error::{UtcpErr, UtcpResult},
    net::{self, NetDeviceHandler},
    platform::{IRQEntry, IRQFlags},
};

static IRQS: Mutex<Vec<IRQEntry>> = Mutex::new(vec![]);

static mut TID: libc::pthread_t = libc::pthread_t::MAX;
static SIGMASK: Mutex<libc::sigset_t> = Mutex::new(unsafe { std::mem::zeroed() });
static mut BARRIER: libc::pthread_barrier_t = unsafe { std::mem::zeroed() };

pub fn intr_request_irq(
    irq: i32,
    handler: fn(irq: i32, NetDeviceHandler),
    flags: IRQFlags,
    name: String,
    dev: NetDeviceHandler,
) -> UtcpResult<()> {
    let mut irqs = IRQS.lock().unwrap();
    // check conflicts
    for ent in &*irqs {
        if ent.irq == irq
            && ent.flags.contains(IRQFlags::SHARED) == flags.contains(IRQFlags::SHARED)
        {
            return Err(UtcpErr::Intr(format!(
                "IRQ {} already registered with the same flags",
                irq
            )));
        }
    }
    // Push to IRQs and update sigmask
    irqs.push(IRQEntry {
        irq,
        handler,
        flags,
        dev,
        debug_name: name.clone(),
    });
    unsafe {
        let mut sigmask = SIGMASK.lock().unwrap();
        libc::sigaddset(&mut *sigmask, irq as c_int);
    }
    log::debug!("registered irq={}, name={}", irq, name);
    Ok(())
}

pub fn intr_init() -> UtcpResult<()> {
    log::debug!("intr init");
    {
        let tid = unsafe { libc::pthread_self() };
        unsafe {
            TID = tid;
        }
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
            // notify the intr thread to handle received packets
            libc::sigaddset(&mut *sigmask, INTR_IRQ_SOFTIRQ);
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
    let err = unsafe { libc::pthread_create(&raw mut TID, null(), intr_thread, ptr::null_mut()) };
    if err != 0 {
        return Err(UtcpErr::Intr(format!("pthread_create failed: {}", err)));
    }
    unsafe { libc::pthread_barrier_wait(&raw mut BARRIER) };

    Ok(())
}

pub fn intr_shutdown() -> UtcpResult<()> {
    unsafe {
        let current_tid = libc::pthread_self();
        if libc::pthread_equal(current_tid, TID) != 0 {
            // thread not created
            return Ok(());
        }
        // Send SIGHUP to notify the intr thread
        libc::pthread_kill(TID, libc::SIGHUP);
        // Wait for the intr thread to exit
        libc::pthread_join(TID, null_mut());
    }
    Ok(())
}

pub fn intr_raise_irq(irq: i32) -> UtcpResult<()> {
    let err = unsafe { libc::pthread_kill(TID, irq) };
    if err != 0 {
        return Err(UtcpErr::Intr(format!("pthread_kill failed: {}", err)));
    }
    Ok(())
}

extern "C" fn intr_thread(_: *mut c_void) -> *mut c_void {
    log::debug!("intr thread start");

    let _ = unsafe { libc::pthread_barrier_wait(&raw mut BARRIER) };

    let mut terminate = false;
    let sigmask = SIGMASK.lock().unwrap();
    let irqs = IRQS.lock().unwrap();
    while !terminate {
        {
            let mut sig_sent = 0;
            let err = unsafe { libc::sigwait(&*sigmask, &mut sig_sent) };
            if err != 0 {
                log::error!("sigwait failed: {}", err);
                break;
            }
            match sig_sent {
                libc::SIGHUP => {
                    terminate = true;
                }
                INTR_IRQ_SOFTIRQ => {
                    net::net_softirq_handler().unwrap();
                }
                _ => {
                    for ent in &*irqs {
                        if ent.irq == sig_sent {
                            (ent.handler)(sig_sent, ent.dev.clone());
                        }
                    }
                }
            }
        }
    }

    log::debug!("intr thread terminated");
    ptr::null_mut()
}
