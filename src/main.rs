use std::env;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::process::ExitCode;

use cccut::{Cutter, Mode};
use cccut::flags::FlagSet;

fn main() -> ExitCode {
    let args = env::args().skip(1);

    let mut fields = 0;

    let mut flag_set = FlagSet::default();
    flag_set.bind_mut_ref("fields", &mut fields, "");

    let files = match flag_set.parse(args) {
        Ok(files) => files,
        Err(err) => {
            println!("Invalid arguments error: {err}");
            return ExitCode::FAILURE;
        }
    };


    let cutter = Cutter::new(Mode::Fields(vec![fields..(fields + 1)], '\t'));
    let mut readers: Vec<Box<dyn BufRead>> = Vec::new();

    for filepath in files {
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