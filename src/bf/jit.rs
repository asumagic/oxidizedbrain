use crate::bf::vm;
use cranelift::prelude::*;
use cranelift_module::{Linkage, Module};
use cranelift_simplejit::{SimpleJITBackend, SimpleJITBuilder};
use std::collections::HashMap;

pub struct Jit {
    builder_context: FunctionBuilderContext,
    ctx: codegen::Context,
    module: Module<SimpleJITBackend>,
}

impl Jit {
    pub fn new() -> Self {
        let builder = SimpleJITBuilder::new(cranelift_module::default_libcall_names());
        let module = Module::new(builder);

        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            module,
        }
    }

    pub fn compile(&mut self, input: &[vm::Op]) -> Result<*const u8, String> {
        self.translate(&input).map_err(|e| e.to_string())?;

        let id = self
            .module
            .declare_function("entry", Linkage::Export, &self.ctx.func.signature)
            .map_err(|e| e.to_string())?;

        self.module
            .define_function(id, &mut self.ctx, &mut codegen::binemit::NullTrapSink {})
            .map_err(|e| e.to_string())?;

        self.module.clear_context(&mut self.ctx);
        self.module.finalize_definitions();

        let code = self.module.get_finalized_function(id);

        Ok(code)
    }

    pub fn translate(&mut self, input: &[vm::Op]) -> Result<(), String> {
        let pointer_type = self.module.target_config().pointer_type();

        self.ctx
            .func
            .signature
            .params
            .push(AbiParam::new(pointer_type));

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);

        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        let tape_pointer = Variable::new(0);
        builder.declare_var(tape_pointer, pointer_type);
        builder.def_var(tape_pointer, builder.block_params(entry_block)[0]);

        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(types::I8));
        let putc = self
            .module
            .declare_function("putchar", Linkage::Import, &sig)
            .expect("problem declaring function");
        let local_putc = self.module.declare_func_in_func(putc, &mut builder.func);

        let mut program_pointer = 0usize;

        let flags = MemFlags::new();

        let mut loop_begin_map = HashMap::<usize, Block>::new();
        let mut loop_end_map = HashMap::<usize, Block>::new();

        while let Some(ins) = input.get(program_pointer) {
            match ins {
                vm::Op::Add { constant } => {
                    let tape_pointer_value = builder.use_var(tape_pointer);
                    let pointed_value = builder.ins().load(types::I8, flags, tape_pointer_value, 0);
                    let constant_value = builder.ins().iconst(types::I8, *constant as i64);
                    let incremented = builder.ins().iadd(pointed_value, constant_value);
                    builder
                        .ins()
                        .store(flags, incremented, tape_pointer_value, 0);
                }
                vm::Op::Set { constant } => {
                    let tape_pointer_value = builder.use_var(tape_pointer);
                    let constant_value = builder.ins().iconst(types::I8, *constant as i64);
                    builder
                        .ins()
                        .store(flags, constant_value, tape_pointer_value, 0);
                }
                vm::Op::Shift { shift_amount } => {
                    let tape_pointer_value = builder.use_var(tape_pointer);
                    let shift_amount_value =
                        builder.ins().iconst(pointer_type, *shift_amount as i64);
                    let new_tape_pointer_value =
                        builder.ins().iadd(tape_pointer_value, shift_amount_value);
                    builder.def_var(tape_pointer, new_tape_pointer_value);
                }
                vm::Op::Write => {
                    let tape_pointer_value = builder.use_var(tape_pointer);
                    let pointed_value = builder.ins().load(types::I8, flags, tape_pointer_value, 0);
                    builder.ins().call(local_putc, &[pointed_value]);
                }
                vm::Op::Read => {
                    unimplemented!();
                }
                vm::Op::JumpIfZero { offset } => {
                    // TODO: should panic if key exists already (it shouldn't)
                    let end_block = loop_end_map
                        .entry(*offset)
                        .or_insert(builder.create_block());

                    let begin_block = loop_begin_map
                        .entry(program_pointer + 1)
                        .or_insert(builder.create_block());

                    let tape_pointer_value = builder.use_var(tape_pointer);
                    let pointed_value = builder.ins().load(types::I8, flags, tape_pointer_value, 0);

                    builder.ins().brz(pointed_value, *end_block, &[]);
                    builder.ins().jump(*begin_block, &[]);
                    builder.switch_to_block(*begin_block);
                }
                vm::Op::JumpIfNonZero { offset } => {
                    let end_block = loop_end_map.get(&(program_pointer + 1)).unwrap();
                    let begin_block = loop_begin_map.get(&(offset)).unwrap();

                    let tape_pointer_value = builder.use_var(tape_pointer);
                    let pointed_value = builder.ins().load(types::I8, flags, tape_pointer_value, 0);

                    builder.ins().brnz(pointed_value, *begin_block, &[]);
                    builder.ins().jump(*end_block, &[]);

                    builder.seal_block(*begin_block);
                    builder.seal_block(*end_block);
                    builder.switch_to_block(*end_block);
                }
                vm::Op::Done => {
                    builder.ins().return_(&[]);
                    builder.finalize();
                }
            }

            program_pointer += 1;
        }

        Ok(())
    }
}
