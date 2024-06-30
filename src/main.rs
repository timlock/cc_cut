use std::env;
use std::fs::File;
use std::io;
use std::io::BufRead;

use cccut::{Cutter, Mode};
use cccut::flags::FlagSet;

fn main() -> Result<(), String> {
    let args = env::args().skip(1);
    let (cutter, files) = create_cutter(args)?;
    run(cutter, files)
}

fn create_cutter(args: impl IntoIterator<Item=String>) -> Result<(Cutter, Vec<String>), String>
{
    let mut flag_set = FlagSet::default();

    let mut fields = 0;
    flag_set.bind_mut_ref("fields", &mut fields, "");
    
    
    let mut delemiter = '\t';
    flag_set.bind_mut_ref("delimiter", &mut delemiter, "");

    let files = match flag_set.parse(args) {
        Ok(files) => files,
        Err(err) => {
            return Err(format!("Invalid arguments error: {err}"));
        }
    };

    let cutter = Cutter::new(Mode::Fields(vec![fields..(fields + 1)], delemiter));

    Ok((cutter, files))
}

fn run(cutter: Cutter, files: Vec<String>) -> Result<(), String> {
    let mut readers: Vec<Box<dyn BufRead>> = Vec::new();

    for filepath in files {
        match File::open(filepath.as_str()) {
            Ok(file) => readers.push(Box::new(io::BufReader::new(file))),
            Err(err) => {
                return Err(format!("Can not open file {filepath}: {err}"));
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
    Ok(())
}