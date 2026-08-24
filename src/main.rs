fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match spis::commands::run(&args) {
        Ok(true) => {}
        Ok(false) => std::process::exit(2),
        Err(e) => {
            eprintln!("error: {e:#}");
            std::process::exit(1);
        }
    }
}
