use std::collections::HashMap;
use std::fmt;
use std::fmt::format;
use std::str;

pub struct FlagSet<'a, V> {
    inner: HashMap<String, &'a mut V>,
}

impl<'a, V> FlagSet<'a, V>
where
    V: str::FromStr,
{
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn register(&mut self, flag: String, value: &'a mut V) {
        self.inner.insert(flag, value);
    }

    pub fn parse(&mut self, args: impl IntoIterator<Item = String>) -> Result<Vec<String>, String>
    where
        <V as std::str::FromStr>::Err: std::fmt::Debug,
    {
        let mut remaining = Vec::new();
        let mut flag = None;

        for arg in args {
            if arg.starts_with('-') {
                flag = arg.strip_prefix('-').map(|s| s.to_owned());
            } else {
                if let Some(key) = flag {
                    if let Some(value) = self.inner.get_mut(&key) {
                        **value = match V::from_str(&arg) {
                            Ok(v) => v,
                            Err(err) => {
                                return Err(format!(
                                    "Could not parse flag {key:?} value {arg}: {err:?}"
                                ))
                            }
                        };
                        // **value = arg;
                    }
                }
                flag = None;
            }
        }

        Ok(remaining)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut flagset = FlagSet::new();

        let mut value = String::new();
        flagset.register(String::from("key"), &mut value);

        let args = vec!["-key".to_string(), "value".to_string()];
        assert!(flagset.parse(args).is_ok());
        assert_eq!(value.as_str(), "value");
    }

    #[test]
    fn test_parse_i32() {
        struct TestCase<V>
        where V: str::FromStr {
            args: Vec<&'static str>,
            expected_flag: (&'static str, Box<dyn impl str::FromStr>),
            expects_err: bool,
        }
        let tests = vec![
            TestCase {
                args: vec!["-i", "1"],
                expected_flag: ("i", Box::new(1)),
                expects_err: false,
            },
            TestCase {
                args: vec!["-b,", "true"],
                expected_flag: ("b", Box::new(true)),
                expects_err: false,
            },
        ];

        for test in tests {
            let mut flag_set = FlagSet::new();

            let mut value = 0;
            flag_set.register(test.expected_flag.0.to_string(), &mut value);

            let result = flag_set.parse(test.args.iter().map(|a| a.to_string()));
            assert_eq!(test.expects_err, result.is_err());

            assert_eq!(test.expected_flag.1, value);
        }
    }
}
