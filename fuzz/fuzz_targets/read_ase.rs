#![no_main]

use libfuzzer_sys::fuzz_target;
extern crate adobe_swatch_exchange;

fuzz_target!(|data: &[u8]| {
    let _ = adobe_swatch_exchange::read_ase(data);
});
