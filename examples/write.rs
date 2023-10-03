#![allow(unused)]
use adobe_swatch_exchange::{ColorBlock, ColorType, ColorValue, Group};

fn main() -> Result<(), std::io::Error> {
    let group = Group::new(
        "Aurora".to_owned(),
        vec![
            ColorBlock::new(
                "#BF616A".to_owned(),
                ColorValue::Rgb(0.749_019_6, 0.380_392_16, 0.415_686_28),
                ColorType::Normal,
            ),
            ColorBlock::new(
                "#D08770".to_owned(),
                ColorValue::Rgb(0.815_686_3, 0.529_411_8, 0.439_215_7),
                ColorType::Normal,
            ),
            ColorBlock::new(
                "#EBCB8B".to_owned(),
                ColorValue::Rgb(0.921_568_63, 0.796_078_44, 0.545_098_07),
                ColorType::Normal,
            ),
            ColorBlock::new(
                "#A3BE8C".to_owned(),
                ColorValue::Rgb(0.639_215_7, 0.745_098_05, 0.549_019_63),
                ColorType::Normal,
            ),
            ColorBlock::new(
                "#B48EAD".to_owned(),
                ColorValue::Rgb(0.705_882_4, 0.556_862_8, 0.678_431_4),
                ColorType::Normal,
            ),
        ],
    );
    let ase = adobe_swatch_exchange::create_ase(vec![group], vec![]);
    std::fs::write("examples/aurora.ase", ase)
}
