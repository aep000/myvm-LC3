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
        match self{
            BR => {

            }, /* branch */
            ADD => {
                let instr = *instr as usize;
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
                let instr = *instr as usize;
                let dest = (instr >> 9) & 0x7;
                let offset = sign_extend((instr & 0x1ff) as u16, 9); //Location in memory at Offset + PC which we are loading from
                context.Reg[dest] = mem_read((context.Reg[RNum!(Registers::RPC)]+offset) as usize,context);
            },     /* load */
            ST => {},     /* store */
            JSR => {},    /* jump register */
            AND => {
                let instr = *instr as usize;
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
                let instr = *instr as usize;
                let dest = (instr >> 9) & 0x7;
                let offset = sign_extend((instr & 0x1ff) as u16, 9); //Location in memory at Offset + PC which we are loading from
                context.Reg[dest] = mem_read((context.Reg[RNum!(Registers::RPC)]+offset) as usize,context);
            },    /* load register */
            STR => {},    /* store register */
            RTI => {},    /* unused */
            NOT => {},    /* bitwise not */
            LDI => {
                let instr = *instr as usize;
                let dest = (instr >> 9) & 0x7;
                let offset = sign_extend((instr & 0x1ff) as u16, 9); //Location in memory at Offset + PC contains the location we are loading from
                context.Reg[dest] = mem_read(mem_read(context.Reg[RNum!(Registers::RPC)] as usize,context) as usize,context);
            },    /* load indirect */
            STI => {},    /* store indirect */
            JMP => {},    /* jump */
            RES => {},    /* reserved (unused) */
            LEA => {},    /* load effective address */
            TRAP => {},    /* execute trap */
            ICOUNT => {},
        }
    }
}
