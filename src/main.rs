use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::process::ExitCode;

use cccut::{Cutter, Parameter};

fn main() -> ExitCode {

    let mut args = env::args().skip(1).collect();

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
        for line in output {
            println!("{line}");
        }
    }

    ExitCode::SUCCESS
}

fn parse_parameters(mut args: &mut VecDeque<String>) -> Result<Vec<Parameter>, String>{

    let mut parameters = Vec::new();

    while args.front().is_some() && args.front().as_ref().unwrap().starts_with("-"){
        let parameter = args.pop_front().unwrap();

        match Parameter::try_from(parameter.as_str()) {
            Ok(param) => parameters.push(param),
            Err(err) => { return Err(err); }
        }
    }

    Ok(parameters)
}