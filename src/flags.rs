use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::rc::Rc;
use std::str::FromStr;


pub trait ParseFlagValue {
    fn parse_from_string(&mut self, s: &str) -> Result<(), String>;
    fn try_activate(&mut self) -> Result<(), String>;
}


impl<T> ParseFlagValue for T
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
            true => Ok(FlagPrefix::Long),
            false => match value.starts_with('-') {
                true => Ok(FlagPrefix::Short),
                false => Err(())
            },
        }
    }
}

impl From<FlagPrefix> for &str {
    fn from(value: FlagPrefix) -> Self {
        match value {
            FlagPrefix::Short => "-",
            FlagPrefix::Long => "--",
        }
    }
}

enum ValueRef<'a> {
    MutRef(&'a mut dyn ParseFlagValue),
    RefCell(Rc<RefCell<dyn ParseFlagValue>>),
}

impl<'a> ValueRef<'a> {
    fn parse_from_string(&mut self, s: &str) -> Result<(), String> {
        match self {
            ValueRef::MutRef(inner) => inner.parse_from_string(s),
            ValueRef::RefCell(inner) => inner.borrow_mut().parse_from_string(s),
        }
    }

    fn try_activate(&mut self) -> Result<(), String> {
        match self {
            ValueRef::MutRef(inner) => inner.try_activate(),
            ValueRef::RefCell(inner) => inner.borrow_mut().try_activate(),
        }
    }
}

struct Flag<'a> {
    name: &'a str,
    inner: ValueRef<'a>,
    usage: &'a str,
}

impl<'a> Flag<'a> {
    fn new(name: &'a str, inner: ValueRef<'a>, usage: &'a str) -> Self {
        Self {
            name,
            inner,
            usage,
        }
    }
}

#[derive(Default)]
pub struct FlagSet<'a> {
    inner: HashMap<&'a str, Flag<'a>>,
}

impl<'a> FlagSet<'a>
{
    pub fn bind_mut_ref(&mut self, flag: &'a str, value: &'a mut dyn ParseFlagValue, usage: &'a str) {
        self.inner.insert(&flag[..1], Flag::new(flag, ValueRef::MutRef(value), usage));
    }

    pub fn bind_ref_cell(&mut self, flag: &'a str, value: Rc<RefCell<dyn ParseFlagValue>>, usage: &'a str) {
        self.inner.insert(&flag[..1], Flag::new(flag, ValueRef::RefCell(value), usage));
    }

    pub fn parse(&mut self, args: impl IntoIterator<Item=String>) -> Result<Vec<String>, String>
    {
        let mut remaining = Vec::new();
        let mut flag = None;
        let mut all_flags_parsed = false;

        for arg in args {
            if all_flags_parsed {
                remaining.push(arg);
                continue;
            }

            let prefix = FlagPrefix::try_from(arg.as_str()).ok();

            match prefix {
                Some(FlagPrefix::Long) => {
                    let p: &'static str = FlagPrefix::Long.into();
                    let name = arg.strip_prefix(p).unwrap();
                    flag = Some(name.to_string());
                    if let Some(f) = self.inner.get_mut(name) {
                        if f.inner.try_activate().is_ok() {
                            flag = None;
                        }
                    }
                }
                Some(FlagPrefix::Short) => {
                    let p: &'static str = FlagPrefix::Short.into();
                    let name = arg.strip_prefix(p).unwrap();

                    if name.len() == 1 {
                        flag = Some(name.to_string());
                        if let Some(f) = self.inner.get_mut(name) {
                            if f.inner.try_activate().is_ok() {
                                flag = None;
                            }
                        }
                    } else {
                        for f in name.chars() {
                            if let Some(f) = self.inner.get_mut(f.to_string().as_str()) {
                                if f.inner.try_activate().is_ok() {
                                    flag = None;
                                }
                            }
                        }
                    }
                }
                None => {
                    match flag {
                        Some(flag) => {
                            if let Some(value) = self.inner.get_mut(flag.as_str()) {
                                value.inner
                                    .parse_from_string(&arg)
                                    .map_err(|err| format!("Could not parse flag {flag} err: {err}"))?;
                            }
                        }
                        None => {
                            all_flags_parsed = true;
                            remaining.push(arg);
                        }
                    };
                    flag = None;
                }
            };
        }

        Ok(remaining)
    }

    pub fn print_usage(&self) {
        for (name, flag) in &self.inner {
            println!("{}\n\t{}", name, flag.usage)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCase<V>
        where V: FromStr
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
            flag_set.bind_mut_ref(test.expected_flag.0, &mut value, "");

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
            flag_set.bind_mut_ref(test.expected_flag.0, &mut value, "");
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
            flag_set.bind_mut_ref(test.expected_flag.0, &mut value, "");

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
            flag_set.bind_mut_ref(test.expected_flag1.0, &mut value1, "");
            let mut value2 = false;
            flag_set.bind_mut_ref(test.expected_flag2.0, &mut value2, "");

            let result = flag_set.parse(test.args.iter().map(|a| a.to_string()));

            assert_eq!(test.expected_flag1.1, value1);
            assert_eq!(test.expected_flag2.1, value2);
        }
    }

    #[test]
    fn test_parse_ref_cell() {
        let tests = vec![
            TestCase {
                args: vec!["-i", "1", "-test", "text"],
                expected_flag: ("i", 1),
                expects_err: false,
            },
        ];

        for test in tests {
            let mut flag_set = FlagSet::default();

            let value = Rc::new(RefCell::new(0));
            flag_set.bind_ref_cell(test.expected_flag.0, value.clone(), "");
            let result = flag_set.parse(test.args.iter().map(|a| a.to_string()));
            assert_eq!(test.expects_err, result.is_err());

            assert_eq!(test.expected_flag.1, *value.borrow());
        }
    }

    #[test]
    fn test_parse_remaining() {
        struct TestCase {
            args: Vec<&'static str>,
            expected_flags: Vec<(&'static str, String)>,
            remaining: Vec<&'static str>,
        }
        let tests = vec![
            TestCase {
                args: vec!["--test", "text", "remaining"],
                expected_flags: vec![("test", String::from("text"))],
                remaining: vec!["remaining"],
            },
            TestCase {
                args: vec!["--test", "text", "first", "second", "third"],
                expected_flags: vec![("test", String::from("text"))],
                remaining: vec!["first", "second", "third"],
            },
        ];

        for test in tests {
            let mut flag_set = FlagSet::default();

            let mut actual = Vec::new();
            for (name, _) in &test.expected_flags {
                let value = Rc::new(RefCell::new(String::new()));
                actual.push(value.clone());

                flag_set.bind_ref_cell(*name, value.clone(), "");
            }

            let result = flag_set.parse(test.args.iter().map(|a| a.to_string()));

            assert_eq!(test.expected_flags.len(), actual.len());
            for i in 0..actual.len() {
                assert_eq!(test.expected_flags[i].1, *actual[i].borrow())
            }

            assert!(result.is_ok());
            let result = result.unwrap();

            assert_eq!(test.remaining.len(), result.len());
            for i in 0..result.len() {
                assert_eq!(test.remaining[i], result[i]);
            }
        }
    }

    #[test]
    fn test_parse_remaining_with_bool_flag() {
        struct TestCase {
            args: Vec<&'static str>,
            expected_flags: Vec<(&'static str, bool)>,
            remaining: Vec<&'static str>,
        }
        let tests = vec![
            TestCase {
                args: vec!["--test", "first", "second", "third"],
                expected_flags: vec![("test", true)],
                remaining: vec!["first", "second", "third"],
            },
        ];

        for test in tests {
            let mut flag_set = FlagSet::default();

            let mut actual = Vec::new();
            for (name, _) in &test.expected_flags {
                let value = Rc::new(RefCell::new(false));
                actual.push(value.clone());

                flag_set.bind_ref_cell(*name, value.clone(), "");
            }

            let result = flag_set.parse(test.args.iter().map(|a| a.to_string()));

            assert_eq!(test.expected_flags.len(), actual.len());
            for i in 0..actual.len() {
                assert_eq!(test.expected_flags[i].1, *actual[i].borrow())
            }

            assert!(result.is_ok());
            let result = result.unwrap();

            assert_eq!(test.remaining.len(), result.len());
            for i in 0..result.len() {
                assert_eq!(test.remaining[i], result[i]);
            }
        }
    }
}
