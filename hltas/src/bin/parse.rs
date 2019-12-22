extern crate hltas_rs;

use std::{env::args, fs::read_to_string};

use nom::Err;

use hltas_rs::read::hltas;

fn main() {
    let filename = args().nth(1).unwrap();
    let contents = read_to_string(filename).unwrap();
    match hltas(&contents) {
        Ok((_, hltas)) => println!("{:#?}", hltas),
        Err(Err::Error(mut e)) | Err(Err::Failure(mut e)) => {
            e.whole_input = &contents;
            println!("{}", e)
        }
        _ => unreachable!(),
    }
}
