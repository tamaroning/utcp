pub mod dummy;
pub mod loopback;

const SIGRTMIN: i32 = 34;
const INTR_IRQ_BASE: i32 = SIGRTMIN + 1;
