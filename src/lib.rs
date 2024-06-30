use std::io::BufRead;
use std::ops::Range;

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