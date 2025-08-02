use twine_scheme::repl::Repl;

fn main() {
    let mut repl = Repl::new();
    if let Err(error) = repl.run() {
        eprintln!("REPL error: {error}");
        std::process::exit(1);
    }
}
