use crate::memory::Context;
use crate::memory::mem_read;
use crate::memory::Registers;

fn sign_extend(x: u16, bit_count:usize) -> u16
{
    let extended = 0;
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

#[derive(Copy, Clone)]
pub enum Conds
{
    FL_POS = 1 << 0, /* P */
    FL_ZRO = 1 << 1, /* Z */
    FL_NEG = 1 << 2, /* N */
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
        match self{
            BR => {
                let conds = ((instr >> 9) & 0x7) as u16;
                let offset = sign_extend((instr & 0x1ff) as u16, 9);
                if(conds & context.Reg[RNum!(Registers::RCOND)] == conds){
                    context.Reg[RNum!(Registers::RPC)] = context.Reg[RNum!(Registers::RPC)] + offset;
                }
            }, /* branch */
            ADD => {

                let dest = (instr >> 9) & 0x7;
                let r1 = (instr >> 6) & 0x7; //first source reg
                let immediatMode = (instr >> 5) & 0x1;

                if (immediatMode == 1)
                {
                    let imm5 = sign_extend((instr & 0x1F) as u16, 5); //Sign extend constant we are adding to register in immediate mode
                    context.Reg[dest] = (context.Reg[r1] as u16) + (imm5 as u16);
                }
                else
                {
                    let r2= instr & 0x7; //second source reg
                    context.Reg[dest] = (context.Reg[r1] as u16) + (context.Reg[r2] as u16);
                }
                update_flags(dest, context);
            },    /* add  */
            LD => {

                let dest = (instr >> 9) & 0x7;
                let offset = sign_extend((instr & 0x1ff) as u16, 9); //Location in memory at Offset + PC which we are loading from
                context.Reg[dest] = mem_read((context.Reg[RNum!(Registers::RPC)]+offset) as usize,context);
                update_flags(dest, context);
            },     /* load */
            ST => {
                let src = (instr >> 9) & 0x7;
                let offset = sign_extend((instr & 0x1ff) as u16, 9);
                context.Memory[(context.Reg[RNum!(Registers::RPC)]+offset) as usize]= context.Reg[src];
            },     /* store */
            JSR => {
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
            AND => {

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
            LDR => {

                let dest = (instr >> 9) & 0x7;
                let baseR = (instr >> 6) & 0x7;
                let offset:u16 = sign_extend((instr & 0x3F) as u16, 6); //Location in memory at Offset + PC which we are loading from
                context.Reg[dest] = mem_read(((context.Reg[baseR] as u16)+offset) as usize,context);
                update_flags(dest, context);
            },    /* load register */
            STR => {
                let src = (instr >> 9) & 0x7;
                let dest = (instr >> 6) & 0x7;
                let offset = sign_extend((instr & 0x3F) as u16, 6);
                context.Memory[(context.Reg[dest]+offset) as usize]= context.Reg[src];
            },    /* store register */
            RTI => {},    /* unused */
            NOT => {
                let dest = (instr >> 9) & 0x7;
                let src = (instr >> 6) & 0x7;
                context.Reg[dest] = !context.Reg[src];
                update_flags(dest, context);
            },    /* bitwise not */
            LDI => {

                let dest = (instr >> 9) & 0x7;
                let offset = sign_extend((instr & 0x1ff) as u16, 9); //Location in memory at Offset + PC contains the location we are loading from
                context.Reg[dest] = mem_read(mem_read(context.Reg[RNum!(Registers::RPC)] as usize,context) as usize,context);
                update_flags(dest, context);
            },    /* load indirect */
            STI => {
                let src = (instr >> 9) & 0x7;
                let offset = sign_extend((instr & 0x1ff) as u16, 9);
                context.Memory[context.Memory[(context.Reg[RNum!(Registers::RPC)]+offset) as usize] as usize]= context.Reg[src];
            },    /* store indirect */
            JMP => {
                let src = (instr >> 6) & 0x7;
                context.Reg[RNum!(Registers::RPC)] = context.Reg[src];
            },    /* jump */
            RES => {},    /* reserved (unused) */
            LEA => {
                let dest = (instr >> 9) & 0x7;
                let offset = sign_extend((instr & 0x1ff) as u16, 9);
                context.Reg[dest] = context.Reg[RNum!(Registers::RPC)] + offset;
                update_flags(dest, context);
            },    /* load effective address */
            TRAP => {},    /* execute trap */
            ICOUNT => (),
        }
    }
}
