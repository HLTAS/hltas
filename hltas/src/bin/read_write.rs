extern crate hltas_rs;

use std::{
    env::args,
    fs::{read_to_string, File},
};

use nom::Err;

use hltas_rs::{read, write};

fn main() {
    let input_filename = args().nth(1).unwrap();
    let output_filename = args().nth(2).unwrap();
    let contents = read_to_string(input_filename).unwrap();

    match read::hltas(&contents) {
        Ok((_, hltas)) => {
            let output_file = File::create(output_filename).unwrap();
            println!("{:#?}", write::hltas(output_file, &hltas));
        }
        Err(Err::Error(e)) | Err(Err::Failure(e)) => println!("{:#?}", e),
        _ => unreachable!(),
    }
}
