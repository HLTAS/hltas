extern crate hltas;

use std::{env::args, fs::read_to_string};

use hltas::HLTAS;

fn main() {
    let filename = args().nth(1).unwrap();
    let contents = read_to_string(filename).unwrap();
    match HLTAS::from_str(&contents) {
        Ok(hltas) => println!("{:#?}", hltas),
        Err(e) => println!("{}", e),
    }
}
