#![no_main]

use libfuzzer_sys::fuzz_target;
extern crate adobe_swatch_exchange;

fuzz_target!(|data: &[u8]| {
    // fuzzes ColorBlock::parse(), skipping the magic bytes in `read_ase()`
    // for this to work, the function has to be temporarily made public
    let _ = adobe_swatch_exchange::ColorBlock::parse(data);
});
