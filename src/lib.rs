#![no_std]

mod gdt;
mod panic;
mod print;
mod terminal;

#[no_mangle]
#[allow(clippy::empty_loop)]
pub extern "C" fn kernel_main() {
    let mut t = terminal::Terminal::new();
    t.flush();
    loop {
        if let Some(key) = terminal::ps2::read_if_ready() {
            t.handle_key(key);
            t.flush();
        }
    }
}
