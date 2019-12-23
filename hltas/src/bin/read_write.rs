extern crate hltas_rs;

use std::{
    env::args,
    fs::{read_to_string, File},
};

use hltas_rs::HLTAS;

fn main() {
    let input_filename = args().nth(1).unwrap();
    let output_filename = args().nth(2).unwrap();
    let contents = read_to_string(input_filename).unwrap();

    match HLTAS::from_str(&contents) {
        Ok(hltas) => {
            let output_file = File::create(output_filename).unwrap();
            println!("{:#?}", hltas.to_writer(output_file));
        }
        Err(e) => println!("{}", e),
    }
}