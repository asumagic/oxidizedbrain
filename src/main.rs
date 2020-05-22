use std::fs;
use std::mem;
use std::num::Wrapping;
use structopt::StructOpt;

mod bf;
use crate::bf::*;

#[derive(Debug, StructOpt)]
struct Cli {
    path: std::path::PathBuf,

    #[structopt(short, long)]
    dump_ir: bool,

    #[structopt(long)]
    no_run: bool,

    #[structopt(long)]
    allow_unsafe: bool,

    #[structopt(long)]
    jit: bool,

    tape_size: Option<usize>,
}

fn main() {
    let args = Cli::from_args();

    let program = bfir::translate(fs::read_to_string(args.path).unwrap().as_bytes()).unwrap();
    let mut tape = vec![Wrapping(0u8); args.tape_size.unwrap_or(30000)];

    if args.dump_ir {
        eprintln!("{:#?}", program);
    }

    if args.no_run {
        return;
    }

    if args.jit {
        let mut jit = jit::Jit::new();
            let entry_point = jit.compile(&program).unwrap();
            let entry_point = unsafe { mem::transmute::<_, fn(*mut u8)>(entry_point) };
            unsafe { entry_point(mem::transmute::<_, *mut u8>(tape.as_mut_ptr())); }
    } else {
        let mut vm = vm::Vm {
            program: &program,
            tape: tape.as_mut_slice(),
        };

        if args.allow_unsafe {
            unsafe {
                vm.run_unchecked();
            }
        } else {
            vm.run().unwrap();
        }
    }
}
