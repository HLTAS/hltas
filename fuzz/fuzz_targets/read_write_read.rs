#![no_main]
use libfuzzer_sys::fuzz_target;

use hltas::{read, write};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok((_, hltas)) = read::hltas(s) {
            let mut output = Vec::new();
            write::hltas(&mut output, &hltas).unwrap();
            let output = std::str::from_utf8(&output).unwrap();
            let (_, hltas_2) = read::hltas(&output).unwrap();
            assert_eq!(hltas, hltas_2);
        }
    }
});
