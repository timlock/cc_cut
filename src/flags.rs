use std::
    collections::HashMap
;

pub struct FlagSet<'a> {
    inner: HashMap<String, &'a mut String>,
}

impl<'a> FlagSet<'a> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn register(&mut self, flag: String, value: &'a mut String) {
        self.inner.insert(flag, value);
    }

    pub fn parse(&mut self, args: impl IntoIterator<Item = String>) -> Vec<String> {
        let mut remaining = Vec::new();
        let mut flag = None;

        for arg in args {
            if arg.starts_with('-'){
                flag = arg.strip_prefix('-').map(|s| s.to_owned());
            }else{
                if let Some(key) = flag {
                    if let Some(value) = self.inner.get_mut(&key) {
                        **value = arg;
                    }
                }
                flag = None;

            }
        }

        remaining
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
        flagset.parse(args);
        assert_eq!(value.as_str(), "value");
    }
}
