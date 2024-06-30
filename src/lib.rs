use std::fmt::Display;
use std::io::BufRead;
use std::ops::Range;
use std::str::FromStr;
use crate::flags::ParseFlagValue;

pub mod flags;

pub enum Mode {
    Characters(Vec<Range<usize>>),
    Bytes(Vec<Range<usize>>),
    Fields(Vec<Range<usize>>, char),
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
            Mode::Fields(ranges, delimiter) => {
                let fields = line.split(*delimiter).collect::<Vec<_>>();

                let mut output = String::new();
                for range in ranges {
                    let range = range.clone();

                    for i in range {
                        if let Some(field) = fields.get(i) {
                            output += " ";
                            output += field;
                        }
                    }
                }
                output
            }
        }
    }
}

#[derive(Default)]
struct List<T> {
    inner: Vec<T>,
}

impl<T> ParseFlagValue for List<T>
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
            let mut actual = List::default();

            let result = actual.parse_from_string(test.args);
            assert!(result.is_ok());

            assert_eq!(test.expected, actual.inner);
        }
    }
}