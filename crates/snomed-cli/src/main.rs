fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match snomed_cli::run(&args) {
        Ok(output) => print!("{output}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
