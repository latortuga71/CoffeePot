#[cfg(test)]
// Note this useful idiom: importing names from outer (for mod tests) scope.
use super::*;

#[test]
fn lui() {
    //00010737                lui     a4,0x10
    let mut cpu = cpu::CPU::new();
    cpu.execute(0x00010737);
    assert_eq!(cpu.x_reg[14], 0x10000);
}
#[test]
fn auipc() {
    // 0000 7197                auipc   gp,0x7
    let mut cpu = cpu::CPU::new();
    cpu.x_reg[0] = 0x17172;
    cpu.pc = 0x10172;
    cpu.execute(0x00007197);
    assert_eq!(cpu.x_reg[3], 0x17172);
}
#[test]
fn c_beqz() {
    // 0x0000000000012164 <+0>:     beqz    a2,0x12230 <memset+204>     let mut cpu = cpu::CPU::new();
    let mut cpu = cpu::CPU::new();
    cpu.pc = 0x12164;
    cpu.x_reg[12] = 130;
    cpu.execute_compressed(0xc671);
    assert_eq!(cpu.pc, 0x12164);
}
#[test]
fn jalr() {
    //1049e:       dee080e7                jalr    -530(ra) # 10288 <__init_libc>t mut cpu = cpu::CPU::new();
    let mut cpu = cpu::CPU::new();
    cpu.pc = 0x1049e;
    cpu.x_reg[1] = 0x1049a;
    cpu.execute(0xdee080e7);
    assert_eq!(cpu.pc, 0x10288);
}
#[test]
fn c_j() {
    //a009                    j       10186 <_start_c>
    let mut cpu = cpu::CPU::new();
    cpu.pc = 0x10184;
    cpu.execute_compressed(0xa009);
    assert_eq!(cpu.pc, 0x10186);
}

#[test]
fn c_mv() {
    //850a                    mv      a0,sp
    let mut cpu = cpu::CPU::new();
    cpu.x_reg[2] = 0x40007ffe70;
    cpu.execute_compressed(0x850a);
    assert_eq!(cpu.x_reg[10], 0x40007ffe70);
}

#[test]
fn addi() {
    // 68e18193                addi    gp,gp,1678 # 17800 <__global_pointer$>
    let mut cpu = cpu::CPU::new();
    cpu.x_reg[3] = 0x17172;
    cpu.execute(0x68e18193);
    assert_eq!(cpu.x_reg[3], 0x17800);
}

#[test]
fn andi() {
    // ff017113                andi    sp,sp,-16
    let mut cpu = cpu::CPU::new();
    cpu.x_reg[2] = 0x40007ffe70;
    cpu.execute(0xff017113);
    assert_eq!(cpu.x_reg[2], 0x40007ffe70);
}
