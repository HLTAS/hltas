#![no_main]
use libfuzzer_sys::fuzz_target;

use hltas::HLTAS;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = HLTAS::from_str(s);
    }
});
