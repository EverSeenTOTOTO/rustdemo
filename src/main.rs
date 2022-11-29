mod macs;

use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct MyError {
    message: String,
}

impl MyError {
    fn new(msg: &str) -> MyError {
        MyError {
            message: msg.to_string(),
        }
    }
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for MyError {
    fn description(&self) -> &str {
        &self.message
    }
}

use macro_demo::nothrow;

struct Demo;

impl Demo {
    #[nothrow]
    fn throw_sth(&self, message: &str) -> Result<(), MyError> {
        Err(MyError::new(message))
    }
}

fn main() {
    let d = Demo {};

    d.throw_sth("throwing").unwrap();
}
