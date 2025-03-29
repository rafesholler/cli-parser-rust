# CLI_Parser
A crate for parsing commands and arguemnts passed to the console.
 
This can parse commands from `std::env::args()` with various arguments. It currently supports optional arguments (both short and long) as well as required parameters.
Note that order is important for required parameters, but not for optional arguments.

## Installation
Run the following Cargo command in your project directory:
```cargo add simple-cli-parser```
Or add the following line to your Cargo.toml:
```simple-cli-parser = "0.1.0"```

## Example Usage
```rust
use simple_cli_parser::*;

fn main() {
    let parser = Parser::new();
    let arg = Arg::new().param("num");
    parser.add_arg(arg);

    let mut args = std::env::args();
    args.next();

    let hashmap = parser.parse(&mut args).unwrap();
    println!("{}", hashmap.get("num"));
}
```