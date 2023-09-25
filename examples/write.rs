use adobe_swatch_exchange::{ColorBlock, ColorType, ColorValue, Group};

fn main() {
    let group = Group::new(
        "Aurora".to_owned(),
        vec![
            ColorBlock::new(
                "#BF616A".to_owned(),
                ColorValue::Rgb(0.7490196078431373, 0.3803921568627451, 0.41568627450980394),
                ColorType::Normal,
            ),
            ColorBlock::new(
                "#D08770".to_owned(),
                ColorValue::Rgb(0.8156862745098039, 0.5294117647058824, 0.4392156862745098),
                ColorType::Normal,
            ),
            ColorBlock::new(
                "#EBCB8B".to_owned(),
                ColorValue::Rgb(0.9215686274509803, 0.796078431372549, 0.5450980392156862),
                ColorType::Normal,
            ),
            ColorBlock::new(
                "#A3BE8C".to_owned(),
                ColorValue::Rgb(0.6392156862745098, 0.7450980392156863, 0.5490196078431373),
                ColorType::Normal,
            ),
            ColorBlock::new(
                "#B48EAD".to_owned(),
                ColorValue::Rgb(0.7058823529411765, 0.5568627450980392, 0.6784313725490196),
                ColorType::Normal,
            ),
        ],
    );
    let _ase = adobe_swatch_exchange::create_ase(vec![group], vec![]);
    // do something with the file, e.g. write it
    //std::fs::write("aurora.ase", ase)
}
