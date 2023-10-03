# Adobe Swatch Exchange
A small, dependencies-less Rust library for writing and reading from `.ase` files.

ASE has the option to either store single colors or colors in a named group. Each color consists of a name, the actual color data (in RGB, CMYK, LAB or Gray format), and a color type, which indicates how the color behaves in the document.

## Usage

```rust
use adobe_swatch_exchange::{ColorBlock, ColorType, ColorValue};

let color = ColorBlock::new(
        "#BF616A".to_owned(),
        ColorValue::Rgb(0.749_019_6, 0.380_392_16, 0.415_686_28),
        ColorType::Normal);

// write colors as ASE
let ase = adobe_swatch_exchange::create_ase(vec![], vec![color]);

// read colors from ASE
let (groups, colors) = adobe_swatch_exchange::read_ase(&*ase).unwrap();
```

## License
This Project is licensed under [MPL-2.0](https://opensource.org/license/mpl-2-0/). It has no affiliation with Adobe Inc.
