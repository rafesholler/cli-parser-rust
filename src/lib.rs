use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
struct InvalidCommandError {
    reason: InvalidCommandReasons
}

impl InvalidCommandError {
    pub fn new(reason: InvalidCommandReasons) -> InvalidCommandError {
        InvalidCommandError { reason }
    }
}

impl Display for InvalidCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.reason {
            InvalidCommandReasons::Unexpected(s) => {
                write!(f, "Invalid command, unexpected token '{}'", s)
            },
            InvalidCommandReasons::Duplicate(s) => {
                write!(f, "Invalid command, duplicate token '{}'", s)
            },
            InvalidCommandReasons::Missing => {
                write!(f, "Invalid command, missing token, perhaps you forgot an argument at the end of the command.")
            }
        }
    }
}

impl Error for InvalidCommandError {
    
}

#[derive(Debug)]
enum InvalidCommandReasons {
    Unexpected(String),
    Missing,
    Duplicate(String),
}

#[derive(Clone, Debug)]
enum ArgTypes {
    Default,
    Short(char),
    None
}

/// Represents a single argument for a Parser.
#[derive(Clone, Debug)]
pub struct Arg {
    name: String,
    arg_type: ArgTypes,
    expecting: bool,
}

impl Arg {
    /// Create a new arg object, note you must call further methods on this for it to be useful.
    pub fn new() -> Arg {
        
        Arg { name: String::new(), arg_type: ArgTypes::None, expecting: false} 
    }

    /// An argument that expects a string of input to follow afterwards.
    pub fn input(self, name: &str) -> Arg {
        Arg { name: String::from(name), arg_type: ArgTypes::Default, expecting: true }
    }

    /// A flag argument, or one that toggles a setting without excpecting another token afterwards.
    pub fn flag(self, name: &str) -> Arg {
        Arg { name: String::from(name), arg_type: self.arg_type, expecting: false }
    }

    /// A short argument, or one that can be called with a single dash and a character as well as the default way.
    pub fn short(self, ch: char) -> Arg {
        Arg { name: self.name, arg_type: ArgTypes::Short(ch), expecting: self.expecting }
    }
}

pub struct Parser {
    args: RefCell<Vec<Arg>>,
}

impl Parser {

    /// Creates a new Parser struct.
    pub fn new() -> Parser {
        Parser { args: RefCell::new(vec![]) }
    }

    /// Adds an argument to the parser.
    pub fn add_arg(&self, arg: Arg) {
        self.args.borrow_mut().push(arg);
    }

    /// Adds a vector of arguments to the parser.
    pub fn add_args(&self, args: &mut Vec<Arg>) {
        self.args.borrow_mut().append(args);
    }


    /// Returns the arguments associated with this parser as a vector.
    pub fn args(&self) -> Vec<Arg> {
        self.args.borrow().clone()
    }

    pub fn len(&self) -> usize {
        self.args.borrow().len()
    }


    /// Parses through the remaining arguments and returns a hashmap of arguments passed and their relevant values.
    pub fn parse(&self, args: &mut impl Iterator<Item = String>) -> Result<HashMap<String, Option<String>>, Box<dyn Error>> {
        let mut hashmap: HashMap<String, Option<String>> = HashMap::new();
        let mut prev_arg: Option<Box<Arg>> = None;
        let mut args = args.peekable();
        let parse_args = self.args.clone().take();

        while let Some(c_arg) = args.next() {
            if c_arg.starts_with("-") {
                // Return error if calling a new argument without providing a follow up argument to the previous one
                if prev_arg.is_some() {
                    return Err(Box::new(InvalidCommandError::new(InvalidCommandReasons::Unexpected(c_arg))));
                }

                if c_arg.starts_with("--") {
                    // Full arg
                    let mut found = false;
                    for arg in &parse_args {
                        if c_arg.ends_with(&arg.name) && c_arg.len() == arg.name.len() + 2 {
                            found = true;
                            if arg.expecting {
                                prev_arg = Some(Box::new(arg.clone()));
                            } else {
                                match hashmap.insert(arg.name.clone(), None) {
                                    Some(_) => return Err(Box::new(InvalidCommandError::new(InvalidCommandReasons::Duplicate(c_arg)))),
                                    None => {},
                                };
                                prev_arg = None;
                            }
                        }
                    }
                    if !found {
                        return Err(Box::new(InvalidCommandError::new(InvalidCommandReasons::Unexpected(c_arg))));
                    }
                } else {
                    // Short arg
                    let mut found = false;
                    for arg in &parse_args {
                        if let ArgTypes::Short(c) = arg.arg_type {
                            if c_arg.ends_with(c) && c_arg.len() == 2 {
                                found = true;
                                if arg.expecting {
                                    prev_arg = Some(Box::new(arg.clone()));
                                } else {
                                    match hashmap.insert(arg.name.clone(), None) {
                                        Some(_) => return Err(Box::new(InvalidCommandError::new(InvalidCommandReasons::Duplicate(c_arg)))),
                                        None => {},
                                    };
                                    prev_arg = None;
                                }
                            }
                        }
                    }
                    if !found {
                        return Err(Box::new(InvalidCommandError::new(InvalidCommandReasons::Unexpected(c_arg))));
                    }
                }
            } else {
                // non-argument token
                if prev_arg.is_none() {
                    return Err(Box::new(InvalidCommandError::new(InvalidCommandReasons::Unexpected(c_arg))));
                }

                match hashmap.insert(prev_arg.unwrap().name, Some(c_arg.clone())) {
                    Some (_) => return Err(Box::new(InvalidCommandError::new(InvalidCommandReasons::Duplicate(c_arg)))),
                    None => {}
                }
                prev_arg = None;
            };

            if args.peek().is_none() && prev_arg.is_some() {
                return Err(Box::new(InvalidCommandError::new(InvalidCommandReasons::Missing)));
            }
        }
        Ok(hashmap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_arg() {
        let default = Arg::new().input("default");
        assert_eq!(default.name, "default");
        assert_eq!(default.expecting, true);
        
        let short = Arg::new().input("short").short('s');
        assert_eq!(short.name, "short");
        if let ArgTypes::Short(c) = short.arg_type {
            assert_eq!(c, 's');
        } else {
            assert!(false);
        }
        assert_eq!(short.expecting, true);

        let flag = Arg::new().flag("flag").short( 'f');
        assert_eq!(flag.expecting, false);
        if let ArgTypes::Short(c) = flag.arg_type {
            assert_eq!(c, 'f');
        } else {
            assert!(false);
        }
        assert_eq!(flag.name, "flag");
    }

    #[test]
    fn append_args() {
        let default = Arg::new().input("default");
        let short = Arg::new().input("short").short('s');
        let flag = Arg::new().flag("flag").short( 'f');

        let parser = Parser::new();
        parser.add_arg(default);
        assert_eq!(parser.len(), 1);
        parser.add_args(&mut vec![short, flag]);
        assert_eq!(parser.len(), 3);
        assert_eq!(parser.args()[2].name, "flag");
    }

    #[test]
    fn test_parse_err() {
        let default = Arg::new().input("default");
        let short = Arg::new().input("short").short('s');
        let flag = Arg::new().flag("flag").short( 'f');

        let parser = Parser::new();
        parser.add_args(&mut vec![default, short, flag]);

        // Tests err on duplicate value
        let mut cmd = "--short short_inp -s s_inp"
            .split_whitespace()
            .map(|s| { String::from(s) });

        let res = parser.parse(&mut cmd);
        assert!(res.is_err());
        
        // Tests err on invalid command order
        let mut cmd = "--default -f"
            .split_whitespace()
            .map(|s| { String::from(s) });

        let res = parser.parse(&mut cmd);
        assert!(res.is_err());

        // Tests err on unexpected token
        let mut cmd = "-f dsjfhkjuh"
        .split_whitespace()
        .map(|s| { String::from(s) });

    let res = parser.parse(&mut cmd);
    assert!(res.is_err());

    // Tests err on missing token
        let mut cmd = "--default"
            .split_whitespace()
            .map(|s| { String::from(s) });

        let res = parser.parse(&mut cmd);
        assert!(res.is_err());
    }

    #[test]
    pub fn test_parse_success() {
        let default = Arg::new().input("default");
        let short = Arg::new().input("short").short('s');
        let flag = Arg::new().flag("flag").short( 'f');

        let parser = Parser::new();
        parser.add_args(&mut vec![default, short, flag]);

        let mut cmd = "--default def_arg -s s_arg -f"
            .split_whitespace()
            .map(|s| { String::from(s) });

        let res = parser.parse(&mut cmd);
        assert!(res.is_ok());

        let res = res.unwrap();
        assert_eq!(res.len(), 3);
        assert!(res.contains_key("default"));
        assert_eq!(res.get("default").unwrap(), &Some(String::from("def_arg")));
        assert!(res.contains_key("short"));
        assert_eq!(res.get("short").unwrap(), &Some(String::from("s_arg")));
        assert!(res.contains_key("flag"));
    }
}