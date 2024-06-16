use std::collections::HashMap;
use std::str;

pub struct FlagSet<'a, V> {
    inner: HashMap<String, &'a mut V>,
}

impl<'a, V> Default for FlagSet<'a, V> {
    fn default() -> Self {
        Self {
            inner: HashMap::new()
        }
    }
}

impl<'a, V> FlagSet<'a, V>
    where
        V: str::FromStr,
{
    pub fn bind(&mut self, flag: String, value: &'a mut V) {
        self.inner.insert(flag, value);
    }

    pub fn parse(&mut self, args: impl IntoIterator<Item=String>) -> Result<Vec<String>, String>
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
                        let parsed = match V::from_str(&arg) {
                            Ok(v) => v,
                            Err(err) => {
                                return Err(format!(
                                    "Could not parse flag {key:?} value {arg}: {err:?}"
                                ));
                            }
                        };
                        **value = parsed;
                    }
                } else {
                    remaining.push(arg);
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
                args: vec!["-i", "1"],
                expected_flag: ("i", 1),
                expects_err: false,
            },
        ];

        for test in tests {
            let mut flag_set = FlagSet::default();

            let mut value = 0;
            flag_set.bind(test.expected_flag.0.to_string(), &mut value);

            let result = flag_set.parse(test.args.iter().map(|a| a.to_string()));
            assert_eq!(test.expects_err, result.is_err());

            assert_eq!(test.expected_flag.1, value);
        }
    }

    #[test]
    fn test_parse_bool() {
        let tests = vec![
            TestCase {
                args: vec!["-b", "true"],
                expected_flag: ("b", true),
                expects_err: false,
            },
            TestCase {
                args: vec!["-b", "fals"],
                expected_flag: ("b", false),
                expects_err: true,
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
}
