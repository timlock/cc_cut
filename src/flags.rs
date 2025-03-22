use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use std::str::FromStr;

pub trait Value {
    fn parse_from_string(&mut self, s: &str) -> Result<(), String>;
    fn try_activate(&mut self) -> Result<(), String>;
}

impl<T> Value for T
where
    T: FromStr + Display,
    <T as FromStr>::Err: Error,
{
    fn parse_from_string(&mut self, arg: &str) -> Result<(), String> {
        match T::from_str(arg) {
            Ok(s) => {
                *self = s;
                Ok(())
            }
            Err(err) => Err(format!("{:?}", err)),
        }
    }

    fn try_activate(&mut self) -> Result<(), String> {
        let t = self.to_string();
        match t.as_str() {
            "true" | "false" => self.parse_from_string("true"),
            _ => Err(String::from("bound value should be of type bool")),
        }
    }
}

enum ValueRef<'a> {
    MutRef(&'a mut dyn Value),
    RefCell(Rc<RefCell<dyn Value>>),
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
    name: &'static str,
    usage: &'static str,
    inner: ValueRef<'a>,
}

impl<'a> Flag<'a> {
    fn new(name: &'static str, inner: ValueRef<'a>, usage: &'static str) -> Self {
        Self { name, inner, usage }
    }
}

fn parse_name(value: &str) -> Option<&str> {
    match value.starts_with("--") {
        true => Some(value.strip_prefix("--").unwrap()),
        false => match value.starts_with('-') {
            true => Some(value.strip_prefix('-').unwrap()),
            false => None,
        },
    }
}

#[derive(Debug)]
pub enum FlagError {
    UnknownFlag(String),
    ParseError((String, String)),
}

impl Display for FlagError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FlagError::UnknownFlag(name) => {
                write!(f, "unknown flag: {name}")
            }
            FlagError::ParseError((name, err)) => {
                write!(f, "could not parse flag {name} err: {err}")
            }
        }
    }
}

#[derive(Default)]
pub struct FlagSet<'a> {
    inner: HashMap<&'static str, Flag<'a>>,
}

impl<'a> FlagSet<'a> {
    pub fn bind_mut_ref(
        &mut self,
        flag: &'static str,
        allow_short: bool,
        value: &'a mut dyn Value,
        usage: &'static str,
    ) {
        let key = if allow_short { &flag[..1] } else { flag };

        let flag = Flag::new(flag, ValueRef::MutRef(value), usage);
        if self.inner.insert(key, flag).is_some() {
            panic!("should not register flag name {key} twice")
        }
    }

    pub fn bind_ref_cell(
        &mut self,
        flag: &'static str,
        allow_short: bool,
        value: Rc<RefCell<dyn Value>>,
        usage: &'static str,
    ) {
        let key = if allow_short { &flag[..1] } else { flag };

        let flag = Flag::new(flag, ValueRef::RefCell(value), usage);
        if self.inner.insert(key, flag).is_some() {
            panic!("should not register flag name {key} twice")
        }
    }

    fn has_flag(&self, name: &str) -> bool {
        if self.inner.get(name).is_some() {
            return true;
        }

        if let Some(flag) = self.inner.get(&name[..1]) {
            return flag.name == name;
        }

        false
    }

    pub fn parse(
        &mut self,
        args: impl IntoIterator<Item = String>,
    ) -> Result<Vec<String>, FlagError> {
        let mut remaining = Vec::new();
        let mut flag: Option<String> = None;
        let mut all_flags_parsed = false;

        for arg in args {
            if arg == "--" {
                all_flags_parsed = true;
            }

            if all_flags_parsed {
                remaining.push(arg);
                continue;
            }

            match flag {
                Some(name) => {
                    if let Some(value) = self.inner.get_mut(name.as_str()) {
                        value
                            .inner
                            .parse_from_string(&arg)
                            .map_err(|err| FlagError::ParseError((name, err)))?;
                    }
                    flag = None;
                }
                None => {
                    let name = parse_name(arg.as_str());
                    match name {
                        Some(name) => {
                            if !self.has_flag(name) {
                                for f in name.chars() {
                                    let short_name = f.to_string();

                                    if !self.has_flag(short_name.as_str()) {
                                        return Err(FlagError::UnknownFlag(name.to_string()));
                                    }

                                    if let Some(value) = self.inner.get_mut(short_name.as_str()) {
                                        if value.inner.try_activate().is_ok() {
                                            flag = None;
                                        }
                                    }
                                }
                            }

                            flag = Some(name.to_string());

                            if let Some(value) = self.inner.get_mut(name) {
                                if value.inner.try_activate().is_ok() {
                                    flag = None;
                                }
                            }
                        }
                        None => {
                            all_flags_parsed = true;
                            remaining.push(arg);
                        }
                    }
                }
            }
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

    #[test]
    fn test_parse_string() {
        let args = vec!["-b", "string"];
        let expected_flag = ("b", String::from("string"));
        let expects_err = false;

        let mut flag_set = FlagSet::default();

        let mut value = String::new();
        flag_set.bind_mut_ref(expected_flag.0, false, &mut value, "");

        let result = flag_set.parse(args.iter().map(|a| a.to_string()));
        assert_eq!(expects_err, result.is_err());

        assert_eq!(expected_flag.1, value);
    }

    #[test]
    fn test_parse_i32() {
        let args = vec!["-i", "1"];
        let expected_flag = ("i", 1);
        let expects_err = false;

        let mut flag_set = FlagSet::default();

        let mut value = 0;
        flag_set.bind_mut_ref(expected_flag.0, false, &mut value, "");
        let result = flag_set.parse(args.iter().map(|a| a.to_string()));
        assert_eq!(expects_err, result.is_err());

        assert_eq!(expected_flag.1, value);
    }

    #[test]
    fn test_parse_bool() {
        let args = vec!["-b"];
        let expected_flag = ("b", true);
        let expects_err = false;

        let mut flag_set = FlagSet::default();

        let mut value = false;
        flag_set.bind_mut_ref(expected_flag.0, false, &mut value, "");

        let result = flag_set.parse(args.iter().map(|a| a.to_string()));
        assert_eq!(expects_err, result.is_err());

        assert_eq!(expected_flag.1, value);
    }

    #[test]
    fn test_parse_multiple_bools() {
        let args = vec!["-ba"];
        let expected_flag1 = ("a", true);
        let expected_flag2 = ("b", true);

        let mut flag_set = FlagSet::default();

        let mut value1 = false;
        flag_set.bind_mut_ref(expected_flag1.0, false, &mut value1, "");
        let mut value2 = false;
        flag_set.bind_mut_ref(expected_flag2.0, false, &mut value2, "");

        let result = flag_set.parse(args.iter().map(|a| a.to_string()));

        assert!(result.is_ok());

        assert_eq!(expected_flag1.1, value1);
        assert_eq!(expected_flag2.1, value2);
    }

    #[test]
    fn test_parse_ref_cell() {
        let args = vec!["-i", "1"];
        let expected_flag = ("i", 1);
        let expects_err = false;

        let mut flag_set = FlagSet::default();

        let value = Rc::new(RefCell::new(0));
        flag_set.bind_ref_cell(expected_flag.0, false, value.clone(), "");
        let result = flag_set.parse(args.iter().map(|a| a.to_string()));
        assert_eq!(expects_err, result.is_err());

        assert_eq!(expected_flag.1, *value.borrow());
    }

    #[test]
    fn test_parse_remaining() {
        let args = vec!["--test", "text", "first", "second", "third"];
        let expected_flags = vec![("test", String::from("text"))];
        let remaining = vec!["first", "second", "third"];

        let mut flag_set = FlagSet::default();

        let mut actual = Vec::new();
        for (name, _) in &expected_flags {
            let value = Rc::new(RefCell::new(String::new()));
            actual.push(value.clone());

            flag_set.bind_ref_cell(*name, false, value.clone(), "");
        }

        let result = flag_set.parse(args.iter().map(|a| a.to_string()));

        assert_eq!(expected_flags.len(), actual.len());
        for i in 0..actual.len() {
            assert_eq!(expected_flags[i].1, *actual[i].borrow())
        }

        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(remaining.len(), result.len());
        for i in 0..result.len() {
            assert_eq!(remaining[i], result[i]);
        }
    }

    #[test]
    fn test_parse_remaining_with_bool_flag() {
        let args = vec!["--test", "first", "second", "third"];
        let expected_flags = vec![("test", true)];
        let remaining = vec!["first", "second", "third"];

        let mut flag_set = FlagSet::default();

        let mut actual = Vec::new();
        for (name, _) in &expected_flags {
            let value = Rc::new(RefCell::new(false));
            actual.push(value.clone());

            flag_set.bind_ref_cell(*name, false, value.clone(), "");
        }

        let result = flag_set.parse(args.iter().map(|a| a.to_string()));

        assert_eq!(expected_flags.len(), actual.len());
        for i in 0..actual.len() {
            assert_eq!(expected_flags[i].1, *actual[i].borrow())
        }

        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(remaining.len(), result.len());
        for i in 0..result.len() {
            assert_eq!(remaining[i], result[i]);
        }
    }
}
