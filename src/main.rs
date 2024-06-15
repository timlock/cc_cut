use std::{env, fs};
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::process::{exit, ExitCode};
use cccut::{Cutter, Parameter};

fn main() -> ExitCode {
    let mut args = env::args();
    let parameters = match parse_parameters(&mut args) {
        Ok(p) => p,
        Err(err) => {
            println!("{err}");
            return ExitCode::FAILURE;
        }
    };
    let cutter = Cutter::new(parameters);
    let mut readers: Vec<Box<dyn BufRead>> = Vec::new();

    for filepath in args {
        match File::open(filepath.as_str()) {
            Ok(file) => readers.push(Box::new(io::BufReader::new(file))),
            Err(err) => {
                println!("Can not open file {filepath}: {err}");
                return ExitCode::FAILURE;
            }
        }
    }

    if readers.is_empty() {
        readers = vec![Box::new(io::BufReader::new(io::stdin()))]
    }

    for reader in readers {
        let output = cutter.cut(reader);
        for line in output{
            println!("{line}");
        }
    }

    ExitCode::SUCCESS
}

fn parse_parameters(args: &mut env::Args) -> Result<Vec<Parameter>, String> {
    let mut parameters = Vec::new();
    for arg in args.take_while(|arg| arg.starts_with('-')) {
        match Parameter::try_from(arg.as_str()) {
            Ok(param) => parameters.push(param),
            Err(err) => { return Err(err); }
        }
    }
    Ok(parameters)
}