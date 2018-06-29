//! CLI tool to use the functions provided by the [cretonne-wasm](../cretonne_wasm/index.html)
//! crate.
//!
//! Reads Wasm binary files, translates the functions' code to Cretonne IR.
#![cfg_attr(feature = "cargo-clippy", allow(too_many_arguments, cyclomatic_complexity))]

use cretonne_codegen::Context;
use cretonne_codegen::print_errors::{pretty_error, pretty_verifier_error};
use cretonne_codegen::settings::FlagsOrIsa;
use cretonne_wasm::{translate_module, DummyEnvironment, ModuleEnvironment};
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;
use term;
use utils::{parse_sets_and_isa, read_to_end};
use wabt::wat2wasm;
use cretonne_codegen::binemit;
use cretonne_codegen::ir;

struct PrintRelocs {
    flag_print: bool,
}

impl binemit::RelocSink for PrintRelocs {
    fn reloc_ebb(
        &mut self,
        where_: binemit::CodeOffset,
        r: binemit::Reloc,
        offset: binemit::CodeOffset,
    ) {
        if self.flag_print {
            println!("reloc_ebb: {} {} at {}", r, offset, where_);
        }
    }

    fn reloc_external(
        &mut self,
        where_: binemit::CodeOffset,
        r: binemit::Reloc,
        name: &ir::ExternalName,
        addend: binemit::Addend,
    ) {
        if self.flag_print {
            println!("reloc_ebb: {} {} {} at {}", r, name, addend, where_);
        }
    }

    fn reloc_jt(&mut self, where_: binemit::CodeOffset, r: binemit::Reloc, jt: ir::JumpTable) {
        if self.flag_print {
            println!("reloc_ebb: {} {} at {}", r, jt, where_);
        }
    }
}

struct PrintTraps {
    flag_print: bool,
}

impl binemit::TrapSink for PrintTraps {
    fn trap(&mut self, offset: binemit::CodeOffset, _srcloc: ir::SourceLoc, code: ir::TrapCode) {
        if self.flag_print {
            println!("trap: {} at {}", code, offset);
        }
    }
}

macro_rules! vprintln {
    ($x: expr, $($tts:tt)*) => {
        if $x {
            println!($($tts)*);
        }
    }
}

macro_rules! vprint {
    ($x: expr, $($tts:tt)*) => {
        if $x {
            print!($($tts)*);
        }
    }
}

pub fn run(
    files: Vec<String>,
    flag_verbose: bool,
    flag_just_decode: bool,
    flag_check_translation: bool,
    flag_print: bool,
    flag_set: &[String],
    flag_isa: &str,
    flag_print_size: bool,
) -> Result<(), String> {
    let parsed = parse_sets_and_isa(flag_set, flag_isa)?;

    for filename in files {
        let path = Path::new(&filename);
        let name = String::from(path.as_os_str().to_string_lossy());
        handle_module(
            flag_verbose,
            flag_just_decode,
            flag_check_translation,
            flag_print,
            flag_print_size,
            &path.to_path_buf(),
            &name,
            parsed.as_fisa(),
        )?;
    }
    Ok(())
}

fn handle_module(
    flag_verbose: bool,
    flag_just_decode: bool,
    flag_check_translation: bool,
    flag_print: bool,
    flag_print_size: bool,
    path: &PathBuf,
    name: &str,
    fisa: FlagsOrIsa,
) -> Result<(), String> {
    let mut terminal = term::stdout().unwrap();
    terminal.fg(term::color::YELLOW).unwrap();
    vprint!(flag_verbose, "Handling: ");
    terminal.reset().unwrap();
    vprintln!(flag_verbose, "\"{}\"", name);
    terminal.fg(term::color::MAGENTA).unwrap();
    vprint!(flag_verbose, "Translating... ");
    terminal.reset().unwrap();

    let mut data = read_to_end(path.clone()).map_err(|err| {
        String::from(err.description())
    })?;

    if !data.starts_with(&[b'\0', b'a', b's', b'm']) {
        data = match wat2wasm(&data) {
            Ok(data) => data,
            Err(e) => return Err(String::from(e.description())),
        };
    }

    let mut dummy_environ = DummyEnvironment::with_flags(fisa.flags.clone());
    translate_module(&data, &mut dummy_environ).map_err(
        |e| e.to_string(),
    )?;

    terminal.fg(term::color::GREEN).unwrap();
    vprintln!(flag_verbose, "ok");
    terminal.reset().unwrap();

    if flag_just_decode {
        if flag_print {
            let num_func_imports = dummy_environ.get_num_func_imports();
            for (def_index, func) in dummy_environ.info.function_bodies.iter().enumerate() {
                let func_index = num_func_imports + def_index;
                let mut context = Context::new();
                context.func = func.clone();
                if let Some(start_func) = dummy_environ.info.start_func {
                    if func_index == start_func {
                        println!("; Selected as wasm start function");
                    }
                }
                vprintln!(flag_verbose, "");
                for export_name in &dummy_environ.info.functions[func_index].export_names {
                    println!("; Exported as \"{}\"", export_name);
                }
                println!("{}", context.func.display(None));
                vprintln!(flag_verbose, "");
            }
            terminal.reset().unwrap();
        }
        return Ok(());
    }

    terminal.fg(term::color::MAGENTA).unwrap();
    if flag_check_translation {
        vprint!(flag_verbose, "Checking... ");
    } else {
        vprint!(flag_verbose, "Compiling... ");
    }
    terminal.reset().unwrap();

    if flag_print_size {
        vprintln!(flag_verbose, "");
    }

    let num_func_imports = dummy_environ.get_num_func_imports();
    let mut total_module_code_size = 0;
    for (def_index, func) in dummy_environ.info.function_bodies.iter().enumerate() {
        let func_index = num_func_imports + def_index;
        let mut context = Context::new();
        context.func = func.clone();

        let mut mem = Vec::new();
        let mut relocs = PrintRelocs { flag_print };
        let mut traps = PrintTraps { flag_print };

        if flag_check_translation {
            context.verify(fisa).map_err(|err| {
                pretty_verifier_error(&context.func, fisa.isa, &err)
            })?;
        } else if let Some(isa) = fisa.isa {
            context.compile_and_emit(isa, &mut mem, &mut relocs, &mut traps).map_err(|err| {
                pretty_error(&context.func, fisa.isa, err)
            })?;
            
            let compiled_size = mem.len();

            if flag_print_size {
                println!(
                    "Function #{} code size: {} bytes",
                    func_index,
                    compiled_size
                );
                total_module_code_size += compiled_size;
                println!(
                    "Function #{} bytecode size: {} bytes",
                    func_index,
                    dummy_environ.func_bytecode_sizes[def_index]
                );
            }
        } else {
            return Err(String::from("compilation requires a target isa"));
        }
        if flag_print {
            vprintln!(flag_verbose, "");
            if let Some(start_func) = dummy_environ.info.start_func {
                if func_index == start_func {
                    println!("; Selected as wasm start function");
                }
            }
            for export_name in &dummy_environ.info.functions[func_index].export_names {
                println!("; Exported as \"{}\"", export_name);
            }
            println!("{}", context.func.display(fisa.isa));
            vprintln!(flag_verbose, "");
        }
    }

    if !flag_check_translation && flag_print_size {
        println!("Total module code size: {} bytes", total_module_code_size);
        let total_bytecode_size: usize = dummy_environ.func_bytecode_sizes.iter().sum();
        println!("Total module bytecode size: {} bytes", total_bytecode_size);
    }

    terminal.fg(term::color::GREEN).unwrap();
    vprintln!(flag_verbose, "ok");
    terminal.reset().unwrap();
    Ok(())
}
