use std::fmt::Display;
use std::io::{BufRead, Chain, Read};
use std::ops::Range;
use std::str::FromStr;
use crate::flags::Value;

pub mod flags;

pub enum Mode {
    Characters(Vec<Range<usize>>),
    Bytes(Vec<Range<usize>>),
    Fields(Vec<usize>, char),
}


pub struct Cutter {
    mode: Mode,
}

impl Cutter {
    pub fn new(mode: Mode) -> Self {
        Self { mode }
    }

    pub fn cut(&self, reader: impl BufRead) -> Vec<String> {
        let mut result = Vec::new();

        for line in reader.lines() {
            let remaining = self.filter(&line.unwrap());
            result.push(remaining);
        }

        result
    }

    fn filter(&self, line: &str) -> String {
        match &self.mode {
            Mode::Characters(ranges) => {
                let mut output = String::new();
                let chars = line.chars().collect::<Vec<_>>();

                for range in ranges {
                    let range = range.clone();

                    if let Some(chars) = chars.get(range) {
                        output += " ";
                        let chars = chars.iter().collect::<String>();
                        output += chars.as_str();
                    }
                }

                output
            }
            Mode::Bytes(ranges) => {
                let mut output = String::new();
                let bytes = line.bytes().collect::<Vec<_>>();

                for range in ranges {
                    let range = range.clone();

                    if let Some(bytes) = bytes.get(range) {
                        output += " ";
                        let bytes = String::from_utf8_lossy(bytes);
                        output += &bytes;
                    }
                }

                output
            }
            Mode::Fields(arg_list, delimiter) => {
                let fields = line.split(*delimiter).collect::<Vec<_>>();

                let mut output = String::new();
                for i in arg_list.iter() {
                    if let Some(field) = fields.get(*i - 1) {
                        output += " ";
                        output += field;
                    }
                }
                output
            }
        }
    }
}




#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;
    use super::*;

    #[test]
    fn test_field() -> Result<(), String> {
        let field = vec![2];
        let cutter = Cutter::new(Mode::Fields(field, '\t'));
        let path = Path::new("src").join("testdata").join("sample.tsv");
        let file = File::open(path).map_err(|err| err.to_string())?;
        let bufReader = BufReader::new(file);

        let expected = vec!["f1", "1", "6", "11", "16", "21"];
        let actual = cutter.cut(bufReader);
        assert_eq!(expected, actual);
        Ok(())
    }
}