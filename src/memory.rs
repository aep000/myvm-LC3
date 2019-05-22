#[derive(Copy, Clone)]
pub enum Registers{
    R0=0,
    R1=1,
    R2=2,
    R3=3,
    R4=4,
    R5=5,
    R6=6,
    R7=7,
    RPC=8,
    RCOND=9,
    RCOUNT=10,
}

macro_rules! RNum{
    ($x:expr)=> ($x as usize);
}

pub struct Context{
    pub Memory: [u16; 65535],
    pub Reg: [u16; RNum!(Registers::RCOUNT)],
}
impl Context{
    pub fn new()->Context{
        return Context{
            Memory: [0; 65535],
            Reg: [0; RNum!(Registers::RCOUNT)],
        }
    }
}

pub fn mem_read(address: usize, context: &mut Context) -> u16
{
   //if (address == MR_KBSR)
   //{
    //   if (check_key())
      // {
    //       memory[MR_KBSR] = (1 << 15);
    //       memory[MR_KBDR] = getchar();
    //   }
    //   else
    //   {
    //       memory[MR_KBSR] = 0;
    //   }
   //}
   return context.Memory[address];
}
