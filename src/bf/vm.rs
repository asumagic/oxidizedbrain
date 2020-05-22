use std::num::Wrapping;

#[derive(Copy, Clone, Debug)]
pub enum Op {
    Add { constant: i8 },
    Set { constant: i8 },
    Shift { shift_amount: i8 },
    Write,
    Read,
    JumpIfZero { offset: usize },
    JumpIfNonZero { offset: usize },
    Done,
}

#[derive(Debug)]
pub enum VmError {
    TapeOutOfBounds,
}

pub struct Vm<'a> {
    pub program: &'a [Op],
    pub tape: &'a mut [Wrapping<u8>],
}

// Safe variant
impl Vm<'_> {
    fn tape_get(&self, index: usize) -> Result<&Wrapping<u8>, VmError> {
        self.tape.get(index).ok_or(VmError::TapeOutOfBounds)
    }

    fn tape_get_mut(&mut self, index: usize) -> Result<&mut Wrapping<u8>, VmError> {
        self.tape.get_mut(index).ok_or(VmError::TapeOutOfBounds)
    }

    pub fn run(&mut self) -> Result<(), VmError> {
        let mut tape_pointer = 0usize;
        let mut program_pointer = 0usize;

        loop {
            let ins = self.program.get(program_pointer).unwrap_or_else(|| {
                panic!("Out of bounds access to VM instruction. Done is expected at the end.")
            });

            let mut next_program_pointer = program_pointer + 1;

            match ins {
                Op::Add { constant } => {
                    *self.tape_get_mut(tape_pointer)? += Wrapping(*constant as u8)
                }
                Op::Set { constant } => {
                    *self.tape_get_mut(tape_pointer)? = Wrapping(*constant as u8)
                }
                Op::Shift { shift_amount } => {
                    // FIXME: the whole conversion stuff should not have to be necessary
                    tape_pointer = (tape_pointer as i64 + *shift_amount as i64) as usize
                }
                Op::Write => print!("{}", self.tape_get(tape_pointer)?.0 as char),
                Op::Read => panic!("Unimplemented ',' instruction"),
                Op::JumpIfZero { offset } => {
                    if self.tape_get(tape_pointer)?.0 == 0 {
                        next_program_pointer = *offset;
                    }
                }
                Op::JumpIfNonZero { offset } => {
                    if self.tape_get(tape_pointer)?.0 != 0 {
                        next_program_pointer = *offset;
                    }
                }
                Op::Done => {
                    break;
                }
            }

            program_pointer = next_program_pointer;
        }

        Ok(())
    }
}

// Unsafe variant
impl Vm<'_> {
    unsafe fn tape_get_unchecked(&self, index: usize) -> &Wrapping<u8> {
        self.tape.get_unchecked(index)
    }

    unsafe fn tape_get_unchecked_mut(&mut self, index: usize) -> &mut Wrapping<u8> {
        self.tape.get_unchecked_mut(index)
    }

    pub unsafe fn run_unchecked(&mut self) {
        let mut tape_pointer = 0usize;
        let mut program_pointer = 0usize;

        loop {
            let ins = self.program.get_unchecked(program_pointer);

            match ins {
                Op::Add { constant } => {
                    *self.tape_get_unchecked_mut(tape_pointer) += Wrapping(*constant as u8);
                }
                Op::Set { constant } => {
                    *self.tape_get_unchecked_mut(tape_pointer) = Wrapping(*constant as u8);
                }
                Op::Shift { shift_amount } => {
                    // FIXME: the whole conversion stuff should not have to be necessary
                    tape_pointer = (tape_pointer as i64 + *shift_amount as i64) as usize;
                }
                Op::Write => {
                    print!("{}", self.tape_get_unchecked(tape_pointer).0 as char);
                }
                Op::Read => panic!("Unimplemented ',' instruction"),
                Op::JumpIfZero { offset } => {
                    if self.tape_get_unchecked(tape_pointer).0 == 0 {
                        program_pointer = *offset;
                        continue;
                    }
                }
                Op::JumpIfNonZero { offset } => {
                    if self.tape_get_unchecked(tape_pointer).0 != 0 {
                        program_pointer = *offset;
                        continue;
                    }
                }
                Op::Done => {
                    break;
                }
            }

            program_pointer += 1;
        }
    }
}
