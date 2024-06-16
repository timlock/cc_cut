use std::any::Any;
use std::io::{BufRead, BufReader};

pub mod flags;

pub enum Parameter {
    Fields(usize)
}

impl TryFrom<&str> for Parameter {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if !value.starts_with('-') {
            return Err(format!("Invalid parameter {value}: Parameter should start with '-'"));
        }
        return match value.split_at(2) {
            ("-f", n) => {
                let pos = match n.parse() {
                    Ok(n) => n,
                    Err(err) => { return Err(format!("Can not parse LIST: {err}")); }
                };
                Ok(Parameter::Fields(pos))
            }
            // ("-d", n) => {
            //
            // },
            _ => Err(format!("Unknown parameter {value}"))
        };
    }
}

pub struct Cutter {
    fields: Option<usize>,
    delimiter: String,
}

impl Cutter {
    pub fn new(parameters: Vec<Parameter>) -> Self {
        let mut separator = String::from("\t");
        let mut fields = None;
        for parameter in parameters {
            match parameter {
                Parameter::Fields(i) => fields = Some(i),
            }
        }
        Self { delimiter: separator, fields }
    }

    pub fn cut(&self, mut reader: impl BufRead) -> Vec<String> {
        let mut result = Vec::new();
        for line in reader.lines() {
            let remainder = line.unwrap().split(&self.delimiter).skip(self.fields.unwrap() - 1).next().unwrap().to_string();
            result.push(remainder);
        }
        result
    }
}