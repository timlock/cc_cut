use std::env;
use std::fs::File;
use std::io;
use std::io::{BufRead, Read, stdin};

use cccut::{ArgList, Cutter, Mode};
use cccut::flags::FlagSet;

fn main() -> Result<(), String> {
    let args = env::args().skip(1);
    let (cutter, remaining) = create_cutter(args)?;
    run(cutter, remaining)
}

fn create_cutter(args: impl IntoIterator<Item=String>) -> Result<(Cutter, Vec<String>), String>
{
    let mut flag_set = FlagSet::default();

    let mut fields = ArgList::default();
    flag_set.bind_mut_ref("fields", true, &mut fields, "");


    let mut delemiter = '\t';
    flag_set.bind_mut_ref("delimiter", true, &mut delemiter, "");

    let remaining = match flag_set.parse(args) {
        Ok(files) => files,
        Err(err) => {
            return Err(format!("Invalid arguments error: {err}"));
        }
    };

    let cutter = Cutter::new(Mode::Fields(fields, delemiter));

    Ok((cutter, remaining))
}

fn run(cutter: Cutter, remaining: Vec<String>) -> Result<(), String> {
    let mut readers: Vec<Box<dyn BufRead>> = Vec::new();

    if remaining.is_empty() || (remaining.len() == 1 && remaining[0] == "-") {
        readers = vec![Box::new(io::BufReader::new(stdin()))]
    }

    for filepath in remaining {
        match File::open(filepath.as_str()) {
            Ok(file) => readers.push(Box::new(io::BufReader::new(file))),
            Err(err) => {
                return Err(format!("Can not open file {filepath}: {err}"));
            }
        }
    }

    for reader in readers {
        let output = cutter.cut(reader);
        for line in output {
            println!("{line}");
        }
    }
    Ok(())
}