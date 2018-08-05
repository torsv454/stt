//! The stt (Simple Text Template) crate provides a very simple text template engine.
//!   
//! ```
//! let template = stt::Template::new("Hello $who$!").unwrap();
//! let lookup = stt::SingleLookup::new("who","world");
//! assert_eq!(template.render(&lookup),"Hello world!");
//! ```
#[derive(Debug, PartialEq, Clone)]
enum Fragment {
    Constant(String),
    Variable(String),
}

#[derive(PartialEq, Debug)]
enum Mode {
    Constant,
    Variable,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Template {
    fragments: Vec<Fragment>,
}

pub trait Lookup {
    fn lookup(&self, key: &str) -> Option<&str>;
}

pub struct ConstantLookup {
    value: String,
}

impl ConstantLookup {
    pub fn new(value: String) -> Self {
        ConstantLookup { value }
    }
}

impl Lookup for ConstantLookup {
    fn lookup(&self, _key: &str) -> Option<&str> {
        Some(&self.value)
    }
}

pub struct EmptyLookup {}
impl EmptyLookup {
    pub fn new() -> Self {
        EmptyLookup {}
    }
}

impl Lookup for EmptyLookup {
    fn lookup(&self, _key: &str) -> Option<&str> {
        None
    }
}

pub struct SingleLookup {
    key: String,
    value: String,
}

impl SingleLookup {
    pub fn new(key: &str, value: &str) -> Self {
        SingleLookup {
            key: key.to_string(),
            value: value.to_string(),
        }
    }
}

impl Lookup for SingleLookup {
    fn lookup(&self, key: &str) -> Option<&str> {
        if self.key == *key {
            Some(&self.value)
        } else {
            None
        }
    }
}

pub struct ChainedLookup<'a> {
    lookups: Vec<&'a Lookup>,
}

impl<'a> ChainedLookup<'a> {
    fn new() -> Self {
        ChainedLookup {
            lookups: Vec::new(),
        }
    }

    fn add(&mut self, lookup: &'a Lookup) -> &Self {
        self.lookups.push(lookup);
        self
    }
}

impl<'a> Lookup for ChainedLookup<'a> {
    fn lookup(&self, key: &str) -> Option<&str> {
        for lookup in &self.lookups {
            let value = lookup.lookup(key);
            if value.is_some() {
                return value;
            }
        }
        None
    }
}

#[derive(PartialEq, Debug)]
pub enum ParseError {
    UNTERMINATED_VARIABLE,
}

impl Template {
    pub fn new(spec: &str) -> Result<Template, ParseError> {
        let mut result = Vec::new();
        let mut buf = String::new();
        let mut mode = Mode::Constant;
        for c in spec.chars() {
            match c {
                '$' => match mode {
                    Mode::Constant if buf.len() > 0 => {
                        result.push(Fragment::Constant(buf.drain(..).collect()));
                        mode = Mode::Variable;
                    }
                    Mode::Variable if buf.len() == 0 => {
                        buf.push(c);
                        mode = Mode::Constant;
                    }
                    Mode::Variable => {
                        result.push(Fragment::Variable(buf.drain(..).collect()));
                        mode = Mode::Constant;
                    }
                    _ => mode = Mode::Variable,
                },
                _ => buf.push(c),
            }
        }

        if mode == Mode::Variable {
            Err(ParseError::UNTERMINATED_VARIABLE)
        } else {
            if buf.len() > 0 {
                result.push(Fragment::Constant(buf.drain(..).collect()));
            }

            Ok(Template { fragments: result })
        }
    }

    pub fn set(self, key: &str, value: &str) -> Template {
        self.partial(&SingleLookup::new(key, value))
    }

    pub fn partial(&self, lookup: &Lookup) -> Template {
        let mut fragments = Vec::new();
        for fragment in &self.fragments {
            match fragment {
                Fragment::Variable(ref var) => match lookup.lookup(var) {
                    Some(value) => fragments.push(Fragment::Constant(value.to_owned())),
                    _ => fragments.push(fragment.clone()),
                },
                _ => fragments.push(fragment.clone()),
            }
        }
        Template { fragments }
    }

    pub fn render(&self, lookup: &Lookup) -> String {
        let mut result = String::new();
        for fragment in &self.fragments {
            match fragment {
                Fragment::Constant(text) => result.push_str(text),
                Fragment::Variable(var) => match lookup.lookup(var) {
                    Some(text) => result.push_str(text),
                    _ => (),
                },
            }
        }
        result
    }

    pub fn as_spec(&self) -> String {
        let mut spec = String::new();
        for fragment in &self.fragments {
            match fragment {
                Fragment::Constant(text) => if text == "$" {
                    spec.push_str("$$");
                } else {
                    spec.push_str(text);
                },
                Fragment::Variable(var) => {
                    spec.push('$');
                    spec.push_str(var);
                    spec.push('$');
                }
            }
        }
        spec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_template_yields_empty_string() {
        let template = Template::new("").unwrap();
        assert_eq!(template.render(&EmptyLookup::new()), "");
    }

    #[test]
    fn variable_only_yields_value_of_variable() {
        let template = Template::new("$foo$").unwrap();
        assert_eq!(template.render(&SingleLookup::new("foo", "value")), "value");
    }

    #[test]
    fn variable_wrapped_with_constants() {
        let template = Template::new("Hello $who$!").unwrap();
        assert_eq!(
            template.render(&SingleLookup::new("who", "world")),
            "Hello world!"
        );
    }

    #[test]
    fn chained_lookup() {
        let first = SingleLookup::new("alfa", "beta");
        let second = SingleLookup::new("who", "world");
        let mut lookup = ChainedLookup::new();
        lookup.add(&first);
        lookup.add(&second);

        let template = Template::new("Hello $who$!").unwrap();

        assert_eq!(template.render(&lookup), "Hello world!");
    }

}
