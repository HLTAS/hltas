#![no_main]
use libfuzzer_sys::fuzz_target;

use std::str::from_utf8;

use hltas::HLTAS;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = from_utf8(data) {
        if let Ok(hltas) = HLTAS::from_str(s) {
            let mut output = Vec::new();
            hltas.to_writer(&mut output).unwrap();

            let hltas_2 = HLTAS::from_str(from_utf8(&output).unwrap()).unwrap();
            assert_eq!(hltas, hltas_2);
        }
    }
});
