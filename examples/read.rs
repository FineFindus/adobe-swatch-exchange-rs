fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open("examples/aurora.ase")?;

    let (groups, colors) = adobe_swatch_exchange::read_ase(file)?;
    println!("Groups: {:?}", groups);
    println!("Colors: {:?}", colors);

    Ok(())
}
