use derive_wizard::Wizard;
use std::path::PathBuf;

#[derive(Debug, Wizard)]
struct FileConfig {
    #[prompt("Enter the input file path:")]
    path: PathBuf,
}

fn main() {
    let config = FileConfig::wizard();
    println!("  Input:  {:?}", config.path);
}
