use crate::memory::Context;
use crate::memory::mem_read;
use crate::memory::Registers;
use std::str;
use std::io::{self, Read};
use std::process;

fn sign_extend(x: u16, bit_count:usize) -> u16
{
    let extended = x;
    if (((x >> (bit_count - 1)) & 1)==0) {
        let extended = x | (0xFFFF << bit_count);
    }
    return extended;
}

fn update_flags(r:usize, context: &mut Context)
{
    if (context.Reg[r] == 0)
    {
        context.Reg[RNum!(Registers::RCOND)] = Conds::FL_ZRO as u16;
    }
    else if (context.Reg[r] >> 15 == 1) /* a 1 in the left-most bit indicates negative */
    {
        context.Reg[RNum!(Registers::RCOND)] = Conds::FL_NEG as u16;
    }
    else
    {
        context.Reg[RNum!(Registers::RCOND)] = Conds::FL_POS as u16;
    }
}

pub fn getc() -> char {
    let mut buf: [u8; 1] = [0; 1];
    return match io::stdin().read(&mut buf) {
        Ok(_) => buf[0] as char,
        Err(_) => 255 as char
    }
}

#[derive(Copy, Clone)]
pub enum Conds
{
    FL_POS = 1 << 0, /* P */
    FL_ZRO = 1 << 1, /* Z */
    FL_NEG = 1 << 2, /* N */
}

enum TrapCode
{
    GETC = 0x20,  /* get character from keyboard, not echoed onto the terminal */
    OUT = 0x21,   /* output a character */
    PUTS = 0x22,  /* output a word string */
    IN = 0x23,    /* get character from keyboard, echoed onto the terminal */
    PUTSP = 0x24, /* output a byte string */
    HALT = 0x25   /* halt the program */
}
impl TrapCode{
    pub fn run(&self, context: &mut Context, instr: &usize){
        match self{
            TrapCode::GETC => {
                context.Reg[RNum!(Registers::R0)] = getc() as u16;
            },  /* get character from keyboard, not echoed onto the terminal */
            TrapCode::OUT => {
                print!("{}", context.Reg[RNum!(Registers::R0)] as u8 as char);

            },   /* output a character */
            TrapCode::PUTS => {
                let mut c = context.Reg[RNum!(Registers::R0)];
                while(context.Memory[c as usize]!=0){
                    print!("{}", str::from_utf8(&[context.Memory[c as usize] as u8]).expect("not UTF-8"));
                    c+=1;
                }
            },  /* output a word string */
            TrapCode::IN => {
                println!("Enter Character");
                let c = getc();
                println!("{}", c);
                context.Reg[RNum!(Registers::R0)] = c as u16;
            },    /* get character from keyboard, echoed onto the terminal */
            TrapCode::PUTSP => {

            }, /* output a byte string */
            TrapCode::HALT => {
                println!("\nProgram Halted");
                process::exit(1);
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum Instruction{
    BR = 0, /* branch */
    ADD = 1,    /* add  */
    LD = 2,     /* load */
    ST = 3,     /* store */
    JSR = 4,    /* jump register */
    AND = 5,    /* bitwise and */
    LDR = 6,    /* load register */
    STR = 7,    /* store register */
    RTI = 8,    /* unused */
    NOT = 9,    /* bitwise not */
    LDI = 10,    /* load indirect */
    STI = 11,    /* store indirect */
    JMP = 12,    /* jump */
    RES = 13,    /* reserved (unused) */
    LEA = 14,    /* load effective address */
    TRAP = 15,    /* execute trap */
    ICOUNT = 16,

}
impl Instruction{
    pub fn Run(&self, context: &mut Context, instr: &u16){
        let instr = *instr as usize;
        match *self{
            Instruction::BR => {
                #[cfg(debug_assertions)]
                println!("BR");
                let conds = ((instr >> 9) & 0x7) as u16;
                let offset = sign_extend((instr & 0x1ff) as u16, 9);
                if(conds & context.Reg[RNum!(Registers::RCOND)] == conds){
                    context.Reg[RNum!(Registers::RPC)] = context.Reg[RNum!(Registers::RPC)] + offset;
                }
            }, /* branch */
            Instruction::ADD => {
                #[cfg(debug_assertions)]
                println!("ADD");

                let dest = (instr >> 9) & 0x7;
                let r1 = (instr >> 6) & 0x7; //first source reg
                let immediatMode = (instr >> 5) & 0x1;

                if (immediatMode == 1)
                {
                    let imm5 = sign_extend((instr & 0x1F) as u16, 5); //Sign extend constant we are adding to register in immediate mode
                    context.Reg[dest] = ((context.Reg[r1] as i16) + (imm5 as i16)) as u16;
                }
                else
                {
                    let r2= instr & 0x7; //second source reg
                    //println!("r1:{a},r2:{b},dest:{c}",a=context.Reg[r1],b=context.Reg[r2],c=context.Reg[dest]);
                    context.Reg[dest] = ((context.Reg[r1] as i16) + (context.Reg[r2] as i16)) as u16;
                }
                update_flags(dest, context);
            },    /* add  */
            Instruction::LD => {
                #[cfg(debug_assertions)]
                println!("LD");
                let dest = (instr >> 9) & 0x7;
                let offset = sign_extend((instr & 0x1ff) as u16, 9); //Location in memory at Offset + PC which we are loading from
                context.Reg[dest] = mem_read((context.Reg[RNum!(Registers::RPC)]+offset) as usize,context);
                update_flags(dest, context);
            },     /* load */
            Instruction::ST => {
                #[cfg(debug_assertions)]
                println!("ST");
                let src = (instr >> 9) & 0x7;
                let offset = sign_extend((instr & 0x1ff) as u16, 9);
                context.Memory[(context.Reg[RNum!(Registers::RPC)]+offset) as usize]= context.Reg[src];
            },     /* store */
            Instruction::JSR => {
                #[cfg(debug_assertions)]
                println!("JSR");
                let flag = (instr >> 11) & 0x1;
                context.Reg[RNum!(Registers::R7)] = context.Reg[RNum!(Registers::RPC)];
                if(flag == 0){
                    let src = (instr >> 6) & 0x7;
                    context.Reg[RNum!(Registers::RPC)] = context.Reg[src];
                }
                else{
                    context.Reg[RNum!(Registers::RPC)] = context.Reg[RNum!(Registers::RPC)] + sign_extend((instr & 0x3ff) as u16, 11);
                }
            },    /* jump register */
            Instruction::AND => {
                #[cfg(debug_assertions)]
                println!("AND");
                let dest = (instr >> 9) & 0x7;
                let r1 = (instr >> 6) & 0x7; //first source reg
                let immediatMode = (instr >> 5) & 0x1;

                if (immediatMode == 1)
                {
                    let imm5 = sign_extend((instr & 0x1F) as u16, 5); //Sign extend constant we are adding to register in immediate mode
                    context.Reg[dest] = (context.Reg[r1] as u16) & (imm5 as u16);
                }
                else
                {
                    let r2= instr & 0x7; //second source reg
                    context.Reg[dest] = (context.Reg[r1] as u16) & (context.Reg[r2] as u16);
                }
                update_flags(dest, context);
            },    /* bitwise and */
            Instruction::LDR => {
                #[cfg(debug_assertions)]
                println!("LDR");
                let dest = (instr >> 9) & 0x7;
                let baseR = (instr >> 6) & 0x7;
                let offset:u16 = sign_extend((instr & 0x3F) as u16, 6); //Location in memory at Offset + PC which we are loading from
                context.Reg[dest] = mem_read(((context.Reg[baseR] as u16)+offset) as usize,context);
                update_flags(dest, context);
            },    /* load register */
            Instruction::STR => {
                #[cfg(debug_assertions)]
                println!("STR");
                let src = (instr >> 9) & 0x7;
                let dest = (instr >> 6) & 0x7;
                let offset = sign_extend((instr & 0x3F) as u16, 6);
                context.Memory[(context.Reg[dest]+offset) as usize]= context.Reg[src];
            },    /* store register */
            Instruction::RTI => {},    /* unused */
            Instruction::NOT => {
                #[cfg(debug_assertions)]
                println!("NOT");
                let dest = (instr >> 9) & 0x7;
                let src = (instr >> 6) & 0x7;
                context.Reg[dest] = !context.Reg[src];
                update_flags(dest, context);
            },    /* bitwise not */
            Instruction::LDI => {
                #[cfg(debug_assertions)]
                println!("LDI");
                let dest = (instr >> 9) & 0x7;
                let offset = sign_extend((instr & 0x1ff) as u16, 9); //Location in memory at Offset + PC contains the location we are loading from
                context.Reg[dest] = mem_read(mem_read(context.Reg[RNum!(Registers::RPC)] as usize,context) as usize,context);
                update_flags(dest, context);
            },    /* load indirect */
            Instruction::STI => {
                #[cfg(debug_assertions)]
                println!("STI");
                let src = (instr >> 9) & 0x7;
                let offset = sign_extend((instr & 0x1ff) as u16, 9);
                context.Memory[context.Memory[(context.Reg[RNum!(Registers::RPC)]+offset) as usize] as usize]= context.Reg[src];
            },    /* store indirect */
            Instruction::JMP => {
                #[cfg(debug_assertions)]
                println!("JMP");
                let src = (instr >> 6) & 0x7;
                context.Reg[RNum!(Registers::RPC)] = context.Reg[src];
            },    /* jump */
            Instruction::RES => {},    /* reserved (unused) */
            Instruction::LEA => {
                #[cfg(debug_assertions)]
                println!("LEA");
                let dest = (instr >> 9) & 0x7;
                let offset = sign_extend((instr & 0x1ff) as u16, 9);
                context.Reg[dest] = context.Reg[RNum!(Registers::RPC)] + offset;
                update_flags(dest, context);
            },    /* load effective address */
            Instruction::TRAP => {
                #[cfg(debug_assertions)]
                println!("TRAP");
                let code = instr & 0xFF;
                if code >= 0x20 && code <= 0x25{
                    let code: TrapCode = unsafe { std::mem::transmute(code as i8) };
                    code.run(context, &instr);
                }
            },    /* execute trap */
            Instruction::ICOUNT => (),
        }
    }

    pub fn from_opcode(opcode:u16)->Instruction{
        match opcode{
            0 => Instruction::BR,
            1 => Instruction::ADD,
            2 => Instruction::LD,
            3 => Instruction::ST,
            4 => Instruction::JSR,
            5 => Instruction::AND,
            6 => Instruction::LDR,
            7 => Instruction::STR,
            8 => Instruction::RTI,
            9 => Instruction::NOT,
            10 => Instruction::LDI,
            11 => Instruction::STI,
            12 => Instruction::JMP,
            13 => Instruction::RES,
            14 => Instruction::LEA,
            15 => Instruction::TRAP,
            _ => Instruction::ICOUNT
        }
    }
}
