use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::str;
use std::str::FromStr;

use crate::flags::FlagPrefix::{Long, Short};

pub trait ParseFlag {
    fn parse_from_string(&mut self, s: &str) -> Result<(), String>;
    fn try_activate(&mut self) -> Result<(), String>;
}


impl<T> ParseFlag for T
    where T: FromStr + Display, <T as FromStr>::Err: Debug {
    fn parse_from_string(&mut self, s: &str) -> Result<(), String> {
        match T::from_str(s) {
            Ok(s) => {
                *self = s;
                Ok(())
            }
            Err(err) => {
                Err(format!("{:?}", err))
            }
        }
    }

    fn try_activate(&mut self) -> Result<(), String> {
        let t = self.to_string();
        match t.as_str() {
            "true" | "false" => self.parse_from_string("true"),
            _ => Err(String::from("bound value should be of type bool"))
        }
    }
}


enum FlagPrefix {
    Short,
    Long,
}

impl TryFrom<&str> for FlagPrefix {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.starts_with("--") {
            true => Ok(Long),
            false => match value.starts_with('-') {
                true => Ok(Short),
                false => Err(())
            },
        }
    }
}

impl From<FlagPrefix> for &str {
    fn from(value: FlagPrefix) -> Self {
        match value {
            Short => "-",
            Long => "--",
        }
    }
}

#[derive(Default)]
pub struct FlagSet<'a> {
    inner: HashMap<String, &'a mut dyn ParseFlag>,
}

impl<'a> FlagSet<'a>
{
    pub fn bind(&mut self, flag: String, value: &'a mut dyn ParseFlag) {
        self.inner.insert(flag, value);
    }

    pub fn parse(&mut self, args: impl IntoIterator<Item=String>) -> Result<Vec<String>, String>
    {
        let mut remaining = Vec::new();
        let mut flag = None;

        for arg in args {
            let prefix = FlagPrefix::try_from(arg.as_str()).ok();

            if prefix.is_none() && flag.is_none() {
                remaining.push(arg);
                continue;
            }

            match prefix {
                Some(Long) => {
                    let p: &'static str = Long.into();
                    let name = arg.strip_prefix(p).unwrap();
                    flag = Some(name.to_string());
                    if let Some(f) = self.inner.get_mut(name) {
                        if f.try_activate().is_ok() {
                            flag = None;
                        }
                    }
                }
                Some(Short) => {
                    let p: &'static str = Short.into();
                    let name = arg.strip_prefix(p).unwrap();

                    if name.len() == 1 {
                        flag = Some(name.to_string());
                        if let Some(f) = self.inner.get_mut(name) {
                            if f.try_activate().is_ok() {
                                flag = None;
                            }
                        }
                    } else {
                        for f in name.chars() {
                            if let Some(f) = self.inner.get_mut(&f.to_string()) {
                                if f.try_activate().is_ok() {
                                    flag = None;
                                }
                            }
                        }
                    }
                }
                None => {
                    match flag {
                        Some(flag) => {
                            if let Some(value) = self.inner.get_mut(&flag) {
                                value
                                    .parse_from_string(&arg)
                                    .map_err(|err| format!("Could not parse flag {flag} err: {err}"))?;
                            }
                        }
                        None => {
                            remaining.push(arg);
                        }
                    };
                    flag = None;
                }
            };
        }

        Ok(remaining)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCase<V>
        where V: str::FromStr
    {
        args: Vec<&'static str>,
        expected_flag: (&'static str, V),
        expects_err: bool,
    }

    #[test]
    fn test_parse_string() {
        let tests = vec![
            TestCase {
                args: vec!["-b", "string"],
                expected_flag: ("b", String::from("string")),
                expects_err: false,
            },
            TestCase {
                args: vec!["-b", ""],
                expected_flag: ("b", String::new()),
                expects_err: false,
            },
        ];

        for test in tests {
            let mut flag_set = FlagSet::default();

            let mut value = String::new();
            flag_set.bind(test.expected_flag.0.to_string(), &mut value);

            let result = flag_set.parse(test.args.iter().map(|a| a.to_string()));
            assert_eq!(test.expects_err, result.is_err());

            assert_eq!(test.expected_flag.1, value);
        }
    }

    #[test]
    fn test_parse_i32() {
        let tests = vec![
            TestCase {
                args: vec!["-i", "1", "-test", "text"],
                expected_flag: ("i", 1),
                expects_err: false,
            },
        ];

        for test in tests {
            let mut flag_set = FlagSet::default();

            let mut value = 0;
            flag_set.bind(test.expected_flag.0.to_string(), &mut value);
            let mut x = String::new();
            flag_set.bind("test".to_string(), &mut x);
            let result = flag_set.parse(test.args.iter().map(|a| a.to_string()));
            assert_eq!(test.expects_err, result.is_err());

            assert_eq!(test.expected_flag.1, value);
        }
    }

    #[test]
    fn test_parse_bool() {
        let tests = vec![
            TestCase {
                args: vec!["-b"],
                expected_flag: ("b", true),
                expects_err: false,
            },
        ];

        for test in tests {
            let mut flag_set = FlagSet::default();

            let mut value = false;
            flag_set.bind(test.expected_flag.0.to_string(), &mut value);

            let result = flag_set.parse(test.args.iter().map(|a| a.to_string()));
            assert_eq!(test.expects_err, result.is_err());

            assert_eq!(test.expected_flag.1, value);
        }
    }

    #[test]
    fn test_parse_multiple_bools() {
        struct TestCase {
            args: Vec<&'static str>,
            expected_flag1: (&'static str, bool),
            expected_flag2: (&'static str, bool),
        }
        let tests = vec![
            TestCase {
                args: vec!["-ba"],
                expected_flag1: ("a", true),
                expected_flag2: ("b", true),
            },
        ];

        for test in tests {
            let mut flag_set = FlagSet::default();

            let mut value1 = false;
            flag_set.bind(test.expected_flag1.0.to_string(), &mut value1);
            let mut value2 = false;
            flag_set.bind(test.expected_flag2.0.to_string(), &mut value2);

            let result = flag_set.parse(test.args.iter().map(|a| a.to_string()));

            assert_eq!(test.expected_flag1.1, value1);
            assert_eq!(test.expected_flag2.1, value2);
        }
    }
}
