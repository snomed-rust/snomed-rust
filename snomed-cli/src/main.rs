#![forbid(unsafe_code)]
// Per spec/rust-no-unsafe/index.md: this workspace contains no `unsafe`, and
// the compiler enforces that rather than a grep.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match snomed_cli::run(&args) {
        Ok(output) => print!("{output}"),
        Err(e) => {
            if let Some(failed) = e.downcast_ref::<snomed_cli::ValidationFailed>() {
                print!("{}", failed.report);
            }
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
