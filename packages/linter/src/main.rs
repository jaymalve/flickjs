fn main() -> miette::Result<()> {
    let exit_code = flint::run()?;
    std::process::exit(exit_code);
}
