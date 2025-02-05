pub mod linux;

struct IRQEntry {
    irq: u32,
    flags: u32,
    debug_name: String,
}
