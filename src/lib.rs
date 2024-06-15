use std::any::Any;
use std::io::{BufRead, BufReader};

pub enum Parameter {
    Fields(u32)
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
            _ => Err(format!("Unknown parameter {value}"))
        };
    }
}

pub struct Cutter {
    parameters: Vec<Parameter>,
}

impl Cutter {
    pub fn new(parameters: Vec<Parameter>) -> Self {
        Self { parameters }
    }

    pub fn cut(&self, reader: impl BufRead) -> Vec<String> {

        vec![]
    }
}