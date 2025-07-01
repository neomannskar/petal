use std::fs::File;
use std::io::{Read, Result, Write};
use std::path::Path;

use back::codegen::Generator;
use back::target::Target;
use front::nodes::node::Node;
use front::semantic::{SemanticAnalyzer, SemanticContext};
use front::token::Position;
use middle::ir::{IRContext, IRInstruction};

mod back;
mod config;
mod front;
mod middle;

macro_rules! _here {
    () => {
        println!(
            "Execution passed through here:\n\tfile: {}\n\tline: {}",
            file!(),
            line!()
        )
    };
}

use crate::middle::ir::IRModule;
use crate::middle::spill::perform_spill_pass; // adjust the module path appropriately

fn run_spill_pass_on_module(ir_module: &mut IRModule, available_regs: Vec<String>) {
    // For each function in the module, run the spill pass on its instructions.
    let mut regs = available_regs;

    for func in &mut ir_module.functions {
        let (new_insts, alloc_map) = perform_spill_pass(func.instructions.clone(), &mut regs);
        func.instructions = new_insts;
        // Optionally: log alloc_map information.
        println!(
            "Register allocation for function {}: {:?}",
            func.id, alloc_map
        );
    }
}

fn read_file_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn main() {
    let config = config::PetalConfig::from_args();
    // dbg!(&config);

    let src = match read_file_to_string(Path::new(&config.src)) {
        Ok(s) => s,
        Err(e) => {
            panic!("Error: {}", e);
        }
    };

    // println!("\n{}", src);

    let lexer = front::lexer::Lexer::new(&src);
    let tokens: Vec<(front::token::Token, Position)> = lexer.lex();

    /*
    for (token, _) in &tokens {
        println!("{:?}", token);
    }
    */

    let mut parser =
        front::parser::Parser::new(config.src.clone().to_string_lossy().into_owned(), tokens);
    match parser.parse() {
        Ok(ast) => {
            ast.display(0);
            println!("");

            let analyzer = SemanticAnalyzer::new(ast);

            match analyzer.analyze() {
                Ok(_analyzed_ast) => {
                    println!("Semantic analysis successful!\n");

                    // let mut _code_generator = Generator::new(ir, ir_ctx.target);

                    /*
                    let asm = code_generator.generate();

                    let mut s = config.src.clone().to_string_lossy().into_owned();
                    s.push_str(".s");
                    let mut output_file = File::create(s).unwrap();

                    output_file.write_all(asm.as_bytes()).unwrap();
                    */
                }
                Err(e) => {
                    eprintln!("Semantic analysis failed: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Parsing failed: {}", e);
        }
    }
}
