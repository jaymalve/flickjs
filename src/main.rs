fn main() -> miette::Result<()> {
    let exit_code = zarc::run()?;
    std::process::exit(exit_code);
}
