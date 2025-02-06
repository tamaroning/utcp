pub mod dummy;
pub mod loopback;

const SIGRTMIN: i32 = 34;
const INTR_IRQ_BASE: i32 = SIGRTMIN + 1;

const INTR_IRQ_SIGUSR1: i32 = 10;
pub const INTR_IRQ_SOFTIRQ: i32 = INTR_IRQ_SIGUSR1;
