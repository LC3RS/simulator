pub mod constants;
pub mod enums;
pub mod utils;
pub mod vm;

use vm::Machine;

fn main() {
    let m = Machine::default();
    m.print_registers();
}
