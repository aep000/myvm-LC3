#[macro_use]
mod memory;
mod instructions;

use std::env;
use std::io::Read;
use std::fs::File;
use std::path::Path;

use memory::Registers;
use instructions::Instruction;
use memory::Context;

const PC_START:u16  = 0x3000;


fn get16(mem: &[u8], ind: usize) -> u16 {
    ((mem[ind] as u16) << 8) + mem[ind+1] as u16
}

// https://doc.rust-lang.org/rust-by-example/std_misc/file/open.html
fn read_image(mem: &mut [u16], image_path: &str) -> u32 {
    let path = Path::new(image_path);
     // println!("[*] Loading {}", path.to_str().unwrap());
    let mut file = File::open(&path).expect("Couldn't open file.");

    const SIZE: u32 = std::u16::MAX as u32 * 2 - 2;
    let mut mem_buffer: [u8; SIZE as usize] = [0; SIZE as usize];
    file.read(&mut mem_buffer).expect("Couldn't read file.");
    let length = file.metadata().unwrap().len();
     // println!("[*] File length {}", length);

    let base = get16(&mem_buffer, 0) as usize;
    for i in (2..length).step_by(2) {
        // println!("{}",i);
        mem[base+(i/2 - 1) as usize] = get16(&mem_buffer, i as usize);
    }
    // println!("{:?}", &mem[0x3000..0x4000]);
    length as u32
}

fn main() {
    let mut context: Context = Context::new();
    let args: Vec<String> = env::args().collect();
    for image in args.iter().skip(1) {
        read_image(&mut context.Memory, image);
    }
    context.Reg[RNum!(Registers::RPC)] = PC_START;
    let mut running = true;
    while running{
        let instr: u16 = context.Memory[context.Reg[Registers::RPC as usize] as usize];

        context.Reg[RNum!(Registers::RPC)]+=1;
        let op = instr >> 12;
        if op == 0{
            break;
        }
        //println!("{}",op);
        if(op< Instruction::ICOUNT as u16){
            let op: Instruction = Instruction::fromOpcode(op);
            op.Run(&mut context,&instr);
        }
    }
}
