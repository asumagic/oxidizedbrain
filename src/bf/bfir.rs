use crate::bf::vm;

#[derive(Copy, Clone, Debug)]
enum Op {
    Concrete(vm::Op),
    LoopBegin,
    LoopEnd,
}

struct Builder {
    program: Vec<Op>,
}

#[derive(Debug)]
pub enum Error {
    MismatchedLoopBegin,
    MismatchedLoopEnd,
}

impl Builder {
    fn new() -> Builder {
        Builder {
            program: Vec::<Op>::new(),
        }
    }

    fn emit_add(&mut self, value: i8) {
        if let Some(Op::Concrete(vm::Op::Add { constant })) = self.program.last_mut() {
            *constant += value;
        } else {
            self.program
                .push(Op::Concrete(vm::Op::Add { constant: value }));
        }
    }

    fn emit_shift(&mut self, value: i8) {
        if let Some(Op::Concrete(vm::Op::Shift { shift_amount })) = self.program.last_mut() {
            *shift_amount += value;
        } else {
            self.program.push(Op::Concrete(vm::Op::Shift {
                shift_amount: value,
            }));
        }
    }

    fn optimize(&mut self) -> Result<(), Error> {
        let mut program_pointer = 0usize;

        let mut optimized = Vec::<Op>::with_capacity(self.program.capacity());

        loop {
            let op = self.program.get(program_pointer..self.program.len());

            let mut emit = |slice, source_size: usize| {
                optimized.extend_from_slice(slice);
                program_pointer += source_size;
            };

            match op {
                Some([Op::LoopBegin, Op::Concrete(vm::Op::Add { constant: _ }), Op::LoopEnd, ..]) => {
                    emit(&[Op::Concrete(vm::Op::Set{ constant: 0 })], 3);
                },
                Some([Op::Concrete(vm::Op::Set{constant: a}), Op::Concrete(vm::Op::Add{constant: b})]) => {
                    emit(&[Op::Concrete(vm::Op::Set{ constant: a + b })], 2);
                }
                Some([op, ..]) => { let clone = op.clone(); emit(&[clone], 1) },
                _ => { break; }
            }
        }

        optimized.shrink_to_fit();
        self.program = optimized;

        Ok(())
    }

    fn concretize_loops(&mut self) -> Result<(), Error> {
        let mut offset_stack = Vec::<usize>::new();

        let mut program_pointer = 0usize;

        while let Some(ins) = self.program.get_mut(program_pointer) {
            match ins {
                Op::LoopBegin => offset_stack.push(program_pointer),
                Op::LoopEnd => {
                    let matching_begin = offset_stack.pop().ok_or(Error::MismatchedLoopEnd)?;

                    *self.program.get_mut(matching_begin).unwrap() =
                        Op::Concrete(vm::Op::JumpIfZero {
                            offset: program_pointer + 1,
                        });

                    *self.program.get_mut(program_pointer).unwrap() =
                        Op::Concrete(vm::Op::JumpIfNonZero {
                            offset: matching_begin + 1,
                        });
                }
                _ => (),
            }

            program_pointer += 1;
        }

        if !offset_stack.is_empty() {
            Err(Error::MismatchedLoopBegin)
        } else {
            Ok(())
        }
    }

    fn finalized_program(mut self) -> Result<Vec<vm::Op>, Error> {
        self.optimize()?;
        self.concretize_loops()?;
        self.program.push(Op::Concrete(vm::Op::Done));

        Ok(self
            .program
            .into_iter()
            .map(|ins| match ins {
                Op::Concrete(vm_ins) => vm_ins,
                _ => panic!("expected all compiler instructions to be translated by now"),
            })
            .collect())
    }
}

pub fn translate(source: &[u8]) -> Result<Vec<vm::Op>, Error> {
    let mut ir = Builder::new();

    let mut program_pointer = 0usize;

    while let Some(ins) = source.get(program_pointer) {
        match ins {
            b'+' => ir.emit_add(1),
            b'-' => ir.emit_add(-1),
            b'>' => ir.emit_shift(1),
            b'<' => ir.emit_shift(-1),
            b'.' => ir.program.push(Op::Concrete(vm::Op::Write)),
            b',' => ir.program.push(Op::Concrete(vm::Op::Read)),
            b'[' => ir.program.push(Op::LoopBegin),
            b']' => ir.program.push(Op::LoopEnd),
            _ => (),
        }

        program_pointer += 1;
    }

    ir.finalized_program()
}
