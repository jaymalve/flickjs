fn main() -> miette::Result<()> {
    let exit_code = flick_scan::run()?;
    std::process::exit(exit_code);
}
