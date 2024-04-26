use crate::mmu::{BYTE, WORD,DOUBLE_WORD,HALF};

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


#[test]
fn add() {
    //01c784b3                add     s1,a5,t3
    let mut cpu = cpu::CPU::new();
    cpu.x_reg[28] = 0x100; // t3
    cpu.x_reg[15] = 0x100; // a5
    cpu.x_reg[9] = 0x0; // s1
    cpu.execute(0x1c784b3);
    assert_eq!(cpu.x_reg[9], 0x200);
}

#[test]
fn mul() {
    //039686b3                mul     a3,a3,s9
    let mut cpu = cpu::CPU::new();
    cpu.x_reg[25] = 0xFFFFFFFFFFFFFFFF; // s9 -1
    cpu.x_reg[13] = 0x100; // a5
    cpu.execute(0x039686b3);
    assert_eq!(cpu.x_reg[13], 0xFFFFFFFFFFFFFF00);
}

#[test]
fn sub() {
    //41c90e33                sub     t3,s2,t3
    let mut cpu = cpu::CPU::new();
    cpu.x_reg[28] = 0xFFFFFFFFFFFFFFFF; // t3 -1
    cpu.x_reg[18] = 0x100; // s2
    cpu.execute(0x41c90e33);
    assert_eq!(cpu.x_reg[28], 0x101);
}

#[test]
fn c_li() {
    // 4781                    li      a5,0
    let mut cpu = cpu::CPU::new();
    cpu.x_reg[15] = 0x17172;
    cpu.execute_compressed(0x4781);
    assert_eq!(cpu.x_reg[15], 0x0);
}


#[test]
fn srl() {
    //00b4d5b3                srl     a1,s1,a1
    let mut cpu = cpu::CPU::new();
    cpu.x_reg[11] = 0x4; // a1
    cpu.x_reg[9] = 100; //s1
    cpu.execute(0x00b4d5b3);
    assert_eq!(cpu.x_reg[11], 0x6);
}


#[test]
fn divu() {
    //02b7d7b3                divu    a5,a5,a1
    let mut cpu = cpu::CPU::new();
    cpu.x_reg[11] = 0x4; // a1
    cpu.x_reg[15] = 100; // a5
    cpu.execute(0x02b7d7b3);
    assert_eq!(cpu.x_reg[15], 25);
}

#[test]
fn c_lw() {
    // 410c                    lw      a1,0(a0)
    let mut cpu = cpu::CPU::new();
    cpu.mmu.alloc(0x0, 0x100);
    cpu.mmu.write(0x0, 0x41414141, WORD);
    cpu.x_reg[10] = 0x0; // a0
    // load from address 0x0 a 32bit word into a1
    cpu.execute_compressed(0x410c);
    assert_eq!(cpu.x_reg[11], 0x41414141);
}

#[test]
fn c_sd() {
    // e426                    sd      s1,8(sp)
    let mut cpu = cpu::CPU::new();
    cpu.x_reg[2] = 0x0; // sp
    cpu.x_reg[9] = 0x41414143; // s1
    cpu.mmu.alloc(0x0, 0x100);
    cpu.execute_compressed(0xe426);
    let result = u8::from_le_bytes(cpu.mmu.read(0x8, BYTE).try_into().unwrap());
    // store double word from s1 into address sp + 0x8 
    assert_eq!(result, 0x43);
}

// LOAD FUNCTIONS


#[test]
fn load_word_unsigned() {
    //0007e703                lwu     a4,0(a5)
    let mut cpu = cpu::CPU::new();
    cpu.x_reg[15] = 0x0;
    cpu.x_reg[14] = 0x0;
    cpu.mmu.alloc(0x0, 0x100);
    cpu.mmu.write(0x0, 0x41414141, WORD);
    cpu.execute(0x0007e703);
    assert_eq!(cpu.x_reg[14], 0x41414141);
}

#[test]
fn load_word() {
    // 00132883                lw      a7,1(t1)
    let mut cpu = cpu::CPU::new();
    cpu.x_reg[6] = 0x0;
    cpu.x_reg[17] = 0x0;
    cpu.mmu.alloc(0x0, 0x100);
    cpu.mmu.write(0x1, 0x41414141, WORD);
    cpu.execute(0x00132883);
    assert_eq!(cpu.x_reg[17], 0x41414141);
}

#[test]
fn load_byte() {
    // 00078703                lb      a4,0(a5)
    let mut cpu = cpu::CPU::new();
    cpu.x_reg[15] = 0x0;
    cpu.x_reg[14] = 0x0;
    cpu.mmu.alloc(0x0, 0x100);
    cpu.mmu.write(0x0, 0x41, BYTE);
    cpu.execute(0x00078703);
    assert_eq!(cpu.x_reg[14], 0x41);
}

// 
#[test]
fn load_double_word() {
    // 05843903                ld      s2,88(s0)
    let mut cpu = cpu::CPU::new();
    cpu.x_reg[8] = 0x0; // s0
    cpu.x_reg[18] = 0x0; // s2
    cpu.mmu.alloc(0x0, 0x100);
    cpu.mmu.write(88, 0x4141414141, DOUBLE_WORD);
    cpu.execute(0x05843903);
    assert_eq!(cpu.x_reg[18], 0x4141414141);
}

#[test]
fn load_half() {
    // 00079703                lh      a4,0(a5)
    let mut cpu = cpu::CPU::new();
    cpu.x_reg[15] = 0x0;
    cpu.x_reg[14] = 0x0;
    cpu.mmu.alloc(0x0, 0x100);
    cpu.mmu.write(0x0, 0x4141, HALF);
    cpu.execute(0x00079703);
    assert_eq!(cpu.x_reg[14], 0x4141);
}


// STORE

#[test]
fn store_word() {
    // 0107a423                sw      a6,8(a5)
    let mut cpu = cpu::CPU::new();
    cpu.x_reg[15] = 0x0;
    cpu.x_reg[16] = 0x41414141;
    cpu.mmu.alloc(0x0, 0x100);
    cpu.execute(0x0107a423);
    let value = u32::from_le_bytes(cpu.mmu.read(8, WORD).try_into().unwrap());
    assert_eq!(value, 0x41414141);
}