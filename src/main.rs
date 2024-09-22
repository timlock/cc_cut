use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io;
use std::io::{BufRead, stdin};
use std::str::FromStr;

use cccut::{Cutter, Mode};
use cccut::flags::{FlagSet, Value};

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

    let cutter = Cutter::new(Mode::Fields(fields.inner, delemiter));

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

#[derive(Default)]
pub struct ArgList<T> {
    pub inner: Vec<T>,
}

impl<T> ArgList<T> {
    pub fn new(inner: Vec<T>) -> Self {
        Self { inner }
    }
}


impl<T> Value for ArgList<T>
    where T: FromStr, <T as FromStr>::Err: Display {
    fn parse_from_string(&mut self, arg: &str) -> Result<(), String> {
        let arg = arg.strip_prefix('\"').unwrap_or(arg);
        let arg = arg.strip_suffix('\"').unwrap_or(arg);

        let separator = if arg.contains(',') { ',' } else { ' ' };

        for i in arg.split(separator) {
            match i.parse() {
                Ok(i) => self.inner.push(i),
                Err(err) => return Err(err.to_string())
            }
        }
        Ok(())
    }

    fn try_activate(&mut self) -> Result<(), String> {
        Err(String::from("bound value should be of type bool"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_list() {
        struct TestCase {
            args: &'static str,
            expected: Vec<i32>,
        }
        let tests = vec![
            TestCase {
                args: "1,2,3",
                expected: vec![1, 2, 3],
            },
            TestCase {
                args: "\"1 2 3\"",
                expected: vec![1, 2, 3],
            },
        ];
        for test in tests {
            let mut actual = ArgList::default();

            let result = actual.parse_from_string(test.args);
            assert!(result.is_ok());

            assert_eq!(test.expected, actual.inner);
        }
    }
}