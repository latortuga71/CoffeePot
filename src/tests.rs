#[cfg(test)]
// Note this useful idiom: importing names from outer (for mod tests) scope.
use super::*;

#[test]
fn addi_negative() {
    let mut cpu = cpu::CPU::new();
    cpu.execute(0xff010113);
    assert_eq!(cpu.x_reg[1], 0);
}
#[test]
fn addi() {
    let mut cpu = cpu::CPU::new();
    cpu.execute(0x3e800093);
    assert_eq!(cpu.x_reg[1], 0x3E8);
}

#[test]
fn andi() {
    let mut cpu = cpu::CPU::new();
    cpu.x_reg[1] = 0x3E8;
    cpu.execute(0x7d00f113);
    assert_eq!(cpu.x_reg[2], 0x3C0);
}

#[test]
fn ori() {
    todo!("ORI TEST")
}

#[test]
fn xori() {
    todo!("XORI TEST")
}
