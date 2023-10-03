fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ase_file = std::fs::read("examples/aurora.ase")?;
    let (groups, colors) = adobe_swatch_exchange::read_ase(&*ase_file)?;
    println!("Groups: {:?}", groups);
    println!("Colors: {:?}", colors);
    Ok(())
}
