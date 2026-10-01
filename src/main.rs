fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match spis::commands::run(&args) {
        Ok(true) => {}
        Ok(false) => std::process::exit(2),
        Err(e) => {
            eprintln!("error: {e:#}");
            // 2 for an invocation that is itself wrong, 1 for every other failure.
            std::process::exit(if spis::commands::is_usage(&e) { 2 } else { 1 });
        }
    }
}
