#[macro_use]
mod memory;
mod instructions;

use memory::Registers;
use instructions::Instruction;
use memory::Context;

const PC_START:u16  = 0x3000;


fn main() {
    let mut context: Context = Context::new();
    let mut running = 1;
    context.Reg[RNum!(Registers::RPC)] = PC_START;

    let instr: u16 = context.Reg[Registers::RPC as usize];
    context.Reg[RNum!(Registers::RPC)]+=1;
    let op = instr >> 12;
    if(op< Instruction::ICOUNT as u16){
        let op: Instruction = unsafe { std::mem::transmute(op as i8) };
        op.Run(&mut context,&instr);
    }
}
