//! Vec/List example. Run with: cargo run -p derive-dialoguer-wizard --example vec_lists

use derive_dialoguer_wizard::DialoguerBackend;
use example_surveys::{ShoppingList, StudentGrades};

fn main() -> anyhow::Result<()> {
    let backend = DialoguerBackend::new();
    let shopping = ShoppingList::builder().run(backend)?;
    println!("{shopping:#?}");

    let backend = DialoguerBackend::new();
    let grades = StudentGrades::builder().run(backend)?;
    println!("{grades:#?}");

    Ok(())
}
