//! The stt (Simple Text Template) crate provides a very simple text template engine.
//!   
//! ```
//! let template = stt::Template::new("Hello $who$!");
//! let lookup = |k: &str| if k == "who" {
//!     Some(String::from("world"))
//! } else {
//!     None
//! };
//! assert_eq!(template.render(&lookup),"Hello world!");
//! ```
#[derive(Debug, PartialEq, Clone)]
enum Fragment {
    Constant(String),
    Variable(String),
}

enum Mode {
    Constant,
    Variable,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Template {
    fragments: Vec<Fragment>,
}

impl Template {
    pub fn new(spec: &str) -> Template {
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
        // todo catch unterminated variable
        if buf.len() > 0 {
            result.push(Fragment::Constant(buf.drain(..).collect()));
        }
        println!("{:?}", result);
        Template { fragments: result }
    }

    pub fn set(self, key: &str, value: &str) -> Template {
        self.partial(&|k| {
            if k == key {
                Some(value.to_owned())
            } else {
                None
            }
        })
    }

    pub fn partial<F>(&self, lookup: &F) -> Template
    where
        F: Fn(&str) -> Option<String>,
    {
        let mut fragments = Vec::new();
        for fragment in &self.fragments {
            match fragment {
                Fragment::Variable(ref var) => match lookup(var) {
                    Some(ref value) => fragments.push(Fragment::Constant(value.to_owned())),
                    _ => fragments.push(fragment.clone()),
                },
                _ => fragments.push(fragment.clone()),
            }
        }
        Template { fragments }
    }

    pub fn render<F>(&self, lookup: &F) -> String
    where
        F: Fn(&str) -> Option<String>,
    {
        let mut result = String::new();
        for fragment in &self.fragments {
            match fragment {
                Fragment::Constant(text) => result.push_str(text),
                Fragment::Variable(var) => match lookup(var) {
                    Some(ref text) => result.push_str(text),
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
        assert_eq!(Template::new("").render(&|_k| None), "");
    }

    #[test]
    fn variable_only_yields_value_of_variable() {
        assert_eq!(
            Template::new("$foo$").render(&|k| if k == "foo" {
                Some(String::from("value"))
            } else {
                None
            }),
            "value"
        );
    }

    #[test]
    fn variable_wrapped_with_constants() {
        assert_eq!(
            Template::new("Hello $who$!").render(&|k| if k == "who" {
                Some(String::from("world"))
            } else {
                None
            }),
            "Hello world!"
        );
    }

}
