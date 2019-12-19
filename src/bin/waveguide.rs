extern crate text_io;
extern crate waveguide;

use std::env;
use std::fs;
use std::process;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: waveguide [compile|interpret|[phase]] [path to file]");
        eprintln!("compile: compiles the specified file and outputs the result.");
        eprintln!("interpret: interprets the specified file using the built-in resolver.");
        eprintln!("[phase]: runs compilation of the file up until [phase] of compilation.");
        eprintln!("    phases: parse, structure, resolve, trivialize");
        process::exit(64);
    }

    let code = match fs::read_to_string(&args[2]) {
        Result::Ok(content) => content,
        Result::Err(_err) => {
            eprintln!("Could not read from {}", &args[2]);
            process::exit(74);
        }
    };
    let source_set = waveguide::SourceSet::from_raw_string(&args[2], &code);

    println!("\nStarting...");
    let compile_start = Instant::now();
    match args[1].as_ref() {
        // "compile" => match waveguide::compile(&source_set) {
        //     Result::Ok(program) => println!("{:?}", program.program),
        //     Result::Err(err) => {
        //         eprintln!("{}", err);
        //         process::exit(101);
        //     }
        // },
        "interpret" => unimplemented!(),
        "parse" => match waveguide::parse(&source_set) {
            Result::Ok(program) => println!("{:?}", program),
            Result::Err(err) => {
                eprintln!("{}", err);
                process::exit(101);
            }
        },
        "structure" => match waveguide::structure(&source_set) {
            Result::Ok(program) => println!("{:?}", program),
            Result::Err(err) => {
                eprintln!("{}", err);
                process::exit(101);
            }
        },
        "resolve" => match waveguide::resolve(&source_set) {
            Result::Ok(program) => println!("{:?}", program),
            Result::Err(err) => {
                eprintln!("{}", err);
                process::exit(101);
            }
        },
        // "trivialize" => match waveguide::trivialize(&source_set) {
        //     Result::Ok(program) => println!("{:?}", program),
        //     Result::Err(err) => {
        //         eprintln!("{}", err);
        //         process::exit(101);
        //     }
        // },
        _ => {
            eprintln!(
                "Invalid mode '{}', expected compile, interpret, or a phase.",
                args[1]
            );
            eprintln!("compile: compiles the specified file and outputs the result.");
            eprintln!("interpret: interprets the specified file using the built-in resolver.");
            eprintln!("[phase]: runs compilation of the file up until [phase] of compilation.");
            eprintln!("    phases: parse, structure, resolve, trivialize");
            process::exit(64);
        }
    }
    println!(
        "Task completed sucessfully ({}ms.)\n",
        compile_start.elapsed().as_millis()
    );

    /*
    let mut inputs = Vec::new();
    let entry_point = &program[program.get_entry_point().clone()];
    for input_id in entry_point.borrow_inputs() {
        for (name, id) in entry_point.borrow_symbols() {
            if id == input_id {
                let data_type = program[input_id.clone()].borrow_data_type();
                println!(
                    "Enter data for input '{}' (data type is {:?})",
                    name, data_type
                );
                let final_data;
                loop {
                    print!("> ");
                    io::stdout().flush().unwrap();
                    let line: String = read!("{}\n");
                    // TODO: Handle unclosed brackets and such.
                    match waveguide::util::parse_known_data(&line) {
                        Result::Ok(data) => {
                            if data.matches_data_type(data_type) {
                                final_data = data;
                                break;
                            } else {
                                eprintln!("The variable requires data of type {:?}, but you provided data of an incorrect type.", data_type);
                            }
                        }
                        Result::Err(err) => {
                            eprintln!("An error was encountered while parsing your data:\n{}", err);
                        }
                    }
                }
                inputs.push(final_data);
            }
        }
    }

    println!("\nInterpreting program...");
    let interpret_start = Instant::now();
    let results = match waveguide::interpret(&mut program, inputs, &source_set) {
        Result::Ok(results) => results,
        Result::Err(description) => {
            eprintln!("{}", description);
            process::exit(101);
        }
    };
    println!(
        "Interpretation complete ({}ms.)\n",
        interpret_start.elapsed().as_millis()
    );

    // Have to reborrow it after program was borrowed as mut.
    let entry_point = &program[program.get_entry_point()];
    for (index, output_id) in entry_point.borrow_outputs().iter().enumerate() {
        for (name, id) in entry_point.borrow_symbols() {
            if id == output_id {
                println!("Final value of '{}':", name);
                println!("> {:?}", results[index]);
            }
        }
    }
    */
}
