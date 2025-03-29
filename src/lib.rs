//! A crate for parsing commands and arguemnts passed to the console.
//! 
//! This can parse commands with various arguments.
//! It currently only supports option arguments (or args beginning with a "-"),
//! but an update for regular params and argument rules (like ordering) will be coming soon.
//! 
//! ## Getting Started
//! To start, create a new [`Parser`] struct and add a couple of [`Arg`]s to it using the
//! [`Parser::add_arg()`] or [`Parser::add_args()`] methods.
//! Then, call the [`Parser::parse()`] method with [`std::env::Args`] passed in.
//! 
//! ```
//! use cli_parser::{ Parser, Arg };
//! use std::env;
//! 
//! fn main() {
//!     let parser = Parser::new();
//!     let my_arg = Arg::new().flag("help").short('h');
//!     parser.add_arg(my_arg);
//!     
//!     let mut args = env::args();
//! 
//!     // Don't include the first argument
//!     args.next();
//!     
//!     let hashmap = parser.parse(&mut args).unwrap();
//! 
//!     if hashmap.contains_key("help") {
//!         println!("Help argument called!");
//!     }
//! }
//! ```

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
                write!(f, "Invalid command, missing argument")
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
    Param(bool),
    Input,
    Short(char),
    None
}

/// Represents a single argument which can be passed to a [`Parser`].
/// 
/// # Example
/// ```
/// let arg = Arg::new().param("num");
/// let parser = Parser::new();
/// let mut args = std::env::args();
/// args.next();
/// 
/// parser.add_arg(arg);
/// let output = parser.parse(&mut args).unwrap();
/// ```
#[derive(Clone, Debug)]
pub struct Arg {
    name: String,
    arg_type: ArgTypes,
    expecting: bool,
}

impl Arg {
    /// Create a new arg object, note you must call further methods on this for it to be useful.
    /// 
    /// # Example
    /// ```
    /// let arg = Arg::new();
    /// ```
    pub fn new() -> Arg {
        
        Arg { name: String::new(), arg_type: ArgTypes::None, expecting: false} 
    }

    /// A parameter argument, or one that does not expect any argument to come before it.
    /// Note that the order that these are added to the parser matters.
    /// 
    /// # Example
    /// ```
    /// let arg = Arg::new().param("p1");
    /// ```
    /// `arg` is a required argument and the [`Parser::parse()`] will return an error if it is not present.
    pub fn param(self, name: &str) -> Arg {
        Arg { name: String::from(name), arg_type: ArgTypes::Param(false), expecting: false }
    }

    /// An optional argument that expects a value to follow directly after it.
    /// 
    /// # Example
    /// ```
    /// let arg = Arg::new().input("inp");
    /// ```
    /// Upon parsing, if `--inp` is one of the arguments called, `arg` will be in the output with whatever string comes next in the arguments.
    pub fn input(self, name: &str) -> Arg {
        Arg { name: String::from(name), arg_type: ArgTypes::Input, expecting: true }
    }

    /// A flag argument, or one that toggles a setting without expecting another token afterwards.
    /// 
    /// # Example
    /// ```
    /// let arg = Arg::new().flag("optional");
    /// ```
    /// Upon parsing, if `--optional` is one of the arguments called, `arg` will be in the output with the value `true`.
    pub fn flag(self, name: &str) -> Arg {
        Arg { name: String::from(name), arg_type: self.arg_type, expecting: false }
    }

    /// Sets a short option for the argument, allowing it to be called with a char rather than a string.
    /// 
    /// # Example
    /// ```
    /// let arg = Arg::new().flag("help").short('h');
    /// ```
    /// The `arg` variable can be called by `--help` or by `-h`.
    pub fn short(self, ch: char) -> Arg {
        Arg { name: self.name, arg_type: ArgTypes::Short(ch), expecting: self.expecting }
    }

    fn set_used(&mut self, used: bool) {
        self.arg_type = ArgTypes::Param(used);
    }
}

/// A struct that parses the command line for certain [`Arg`]s.
/// 
/// # Example
/// ```
/// let parser = Parser::new();
/// let arg = Arg::new().param("num");
/// let mut args = std::env::args();
/// args.next();
/// 
/// parser.add_arg(arg);
/// let output = parser.parse(&mut args).unwrap();
/// ```
pub struct Parser {
    args: RefCell<Vec<Arg>>,
}

impl Parser {

    /// Creates a new Parser struct.
    /// 
    /// # Example
    /// ```
    /// let parser = Parser::new();
    /// ```
    pub fn new() -> Parser {
        Parser { args: RefCell::new(vec![]) }
    }

    /// Adds an argument to the parser.
    /// 
    /// # Example
    /// ```
    /// let arg = Arg::new().param("num");
    /// let parser = Parser::new();
    /// 
    /// parser.add_arg(arg);
    /// ```
    pub fn add_arg(&self, arg: Arg) {
        self.args.borrow_mut().push(arg);
    }

    /// Adds a vector of arguments to the parser.
    /// 
    /// # Example
    /// ```
    /// let mut args = vec![
    ///     Arg::new().param("num1"),
    ///     Arg::new().param("num2"),
    /// ];
    /// let parser = Parser::new();
    /// 
    /// parser.add_args(args);
    /// ```
    pub fn add_args(&self, mut args: Vec<Arg>) {
        self.args.borrow_mut().append(&mut args);
    }

    /// Returns the arguments associated with this parser as a vector.
    /// 
    /// # Example
    /// ```
    /// let args = parser.args();
    /// ```
    pub fn args(&self) -> Vec<Arg> {
        self.args.borrow().clone()
    }

    /// Returns the number of arguments associated with this parser.
    /// 
    /// # Example
    /// ```
    /// let n = parser.len();
    /// ```
    pub fn len(&self) -> usize {
        self.args.borrow().len()
    }

    fn get_err(&self, reason: InvalidCommandReasons) -> Result<HashMap<String, Option<String>>, Box<dyn Error>> {
        return Err(Box::new(InvalidCommandError::new(reason)))
    }

    /// Parses through the remaining arguments and returns a hashmap of arguments passed and their relevant values.
    /// 
    /// # Example
    /// ```
    /// let parser = Parser::new();
    /// let mut args = vec![
    ///     Arg::new().param("p1"),
    ///     Arg::new().param("p2"),
    ///     Arg::new().flag("help").short('h');
    /// ];
    /// parser.add_args(args);
    /// 
    /// let mut input_args = std::env::args();
    /// input_args.next();
    /// 
    /// let hashmap = parser.parse(input_args).unwrap();
    /// println!("p1: {}, p2: {}", hashmap.get("p1"), hashmap.get("p2"));
    /// if hashmap.contains_key("help") {
    ///     println!("Help requested!");
    /// }
    /// ```
    pub fn parse(&self, args: &mut impl Iterator<Item = String>) -> Result<HashMap<String, Option<String>>, Box<dyn Error>> {
        let mut hashmap: HashMap<String, Option<String>> = HashMap::new();
        let mut prev_arg: Option<Box<Arg>> = None;
        let mut args = args.peekable();
        let mut parser_args = self.args.clone().take();

        while let Some(c_arg) = args.next() {
            if c_arg.starts_with("-") {
                // Return error if calling a new argument without providing a follow up argument to the previous one
                if prev_arg.is_some() {
                    return self.get_err(InvalidCommandReasons::Unexpected(c_arg));
                }

                if c_arg.starts_with("--") {
                    // Full arg
                    let mut found = false;
                    for arg in &parser_args {
                        if c_arg.ends_with(&arg.name) && c_arg.len() == arg.name.len() + 2 {
                            found = true;
                            if arg.expecting {
                                prev_arg = Some(Box::new(arg.clone()));
                            } else {
                                match hashmap.insert(arg.name.clone(), None) {
                                    Some(_) => return self.get_err(InvalidCommandReasons::Duplicate(c_arg)),
                                    None => {},
                                };
                                prev_arg = None;
                            }
                        }
                    }
                    if !found {
                        return self.get_err(InvalidCommandReasons::Unexpected(c_arg));
                    }
                } else {
                    // Short arg
                    let mut found = false;
                    for arg in &parser_args {
                        if let ArgTypes::Short(c) = arg.arg_type {
                            if c_arg.ends_with(c) && c_arg.len() == 2 {
                                found = true;
                                if arg.expecting {
                                    prev_arg = Some(Box::new(arg.clone()));
                                } else {
                                    match hashmap.insert(arg.name.clone(), None) {
                                        Some(_) => return self.get_err(InvalidCommandReasons::Duplicate(c_arg)),
                                        None => {},
                                    };
                                    prev_arg = None;
                                }
                            }
                        }
                    }
                    if !found {
                        return self.get_err(InvalidCommandReasons::Unexpected(c_arg));
                    }
                }
            } else {
                // non-argument token
                if prev_arg.is_none() {
                    // params
                    let mut found = false;
                    for arg in &mut parser_args {
                        if let ArgTypes::Param(used) = arg.arg_type {
                            if used {
                                continue;
                            }
                            
                            match hashmap.insert(arg.name.clone(), Some(c_arg.clone())) {
                                Some (_) => return self.get_err(InvalidCommandReasons::Duplicate(c_arg)),
                                None => {}
                            };
                            arg.set_used(true);
                            prev_arg = None;
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        return self.get_err(InvalidCommandReasons::Unexpected(c_arg));
                    }
                } else {
                    match hashmap.insert(prev_arg.unwrap().name, Some(c_arg.clone())) {
                        Some (_) => return self.get_err(InvalidCommandReasons::Duplicate(c_arg)),
                        None => {}
                    }
                    prev_arg = None;
                }
            };

            if args.peek().is_none() && prev_arg.is_some() {
                return self.get_err(InvalidCommandReasons::Missing);
            }
        }

        for arg in parser_args {
            if let ArgTypes::Param(false) = arg.arg_type {
                return self.get_err(InvalidCommandReasons::Missing);
            }
        }

        Ok(hashmap)
    }
}



// Tests
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

        let mut param = Arg::new().param("param");
        assert_eq!(param.expecting, false);
        if let ArgTypes::Param(used) = param.arg_type {
            assert!(!used);
        } else {
            assert!(false);
        }
        assert_eq!(param.name, "param");

        param.set_used(true);
        if let ArgTypes::Param(used) = param.arg_type {
            assert!(used);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn append_args() {
        let default = Arg::new().input("default");
        let short = Arg::new().input("short").short('s');
        let flag = Arg::new().flag("flag").short( 'f');

        let parser = Parser::new();
        parser.add_arg(default);
        assert_eq!(parser.len(), 1);
        parser.add_args(vec![short, flag]);
        assert_eq!(parser.len(), 3);
        assert_eq!(parser.args()[2].name, "flag");
    }

    #[test]
    fn test_parse_err() {
        let default = Arg::new().input("default");
        let short = Arg::new().input("short").short('s');
        let flag = Arg::new().flag("flag").short( 'f');
        let param1 = Arg::new().param("p1");
        let param2 = Arg::new().param("p2");

        let parser = Parser::new();
        parser.add_args(vec![default, short, flag, param1, param2]);

        // Tests err on duplicate value
        let mut cmd = "--short short_inp -s s_inp p1 p2"
            .split_whitespace()
            .map(|s| { String::from(s) });

        let res = parser.parse(&mut cmd);
        assert!(res.is_err());
        
        // Tests err on invalid command order
        let mut cmd = "--default -f p1 p2"
            .split_whitespace()
            .map(|s| { String::from(s) });

        let res = parser.parse(&mut cmd);
        assert!(res.is_err());

        // Tests err on unexpected token
        let mut cmd = "p1 p2 -f dsjfhkjuh"
        .split_whitespace()
        .map(|s| { String::from(s) });

        let res = parser.parse(&mut cmd);
        assert!(res.is_err());

        // Tests err on missing token
        let mut cmd = "p1 p2 --default"
            .split_whitespace()
            .map(|s| { String::from(s) });

        let res = parser.parse(&mut cmd);
        assert!(res.is_err());

        // Tests err on missing param
        let mut cmd = "p1 -f"
            .split_whitespace()
            .map(|s| { String::from(s) });

        let res = parser.parse(&mut cmd);
        assert!(res.is_err());

        // Tests err on unrecognized arg
        let mut cmd = "p1 p2 --doesntexist"
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
        let param1 = Arg::new().param("file");
        let param2 = Arg::new().param("path");

        let parser = Parser::new();
        parser.add_args(vec![default, short, flag, param1, param2]);

        let mut cmd = "--default def_arg filename -s s_arg -f pathname"
            .split_whitespace()
            .map(|s| { String::from(s) });

        let res = parser.parse(&mut cmd);
        assert!(res.is_ok());

        let res = res.unwrap();
        assert_eq!(res.len(), 5);
        assert!(res.contains_key("default"));
        assert_eq!(res.get("default").unwrap(), &Some(String::from("def_arg")));
        assert!(res.contains_key("short"));
        assert_eq!(res.get("short").unwrap(), &Some(String::from("s_arg")));
        assert!(res.contains_key("flag"));
        assert_eq!(res.get("flag").unwrap(), &None);
        assert!(res.contains_key("file"));
        assert_eq!(res.get("file").unwrap(), &Some(String::from("filename")));
        assert!(res.contains_key("path"));
        assert_eq!(res.get("path").unwrap(), &Some(String::from("pathname")));
    }
}