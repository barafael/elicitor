//! Vec/List example
//!
//! Run with: cargo run -p derive-requestty-wizard --example vec_lists

use derive_requestty_wizard::RequesttyBackend;
use example_surveys::{ShoppingList, StudentGrades};

fn main() -> anyhow::Result<()> {
    let backend = RequesttyBackend::new();
    let shopping = ShoppingList::builder().run(backend)?;
    println!("{shopping:#?}");

    let backend = RequesttyBackend::new();
    let grades = StudentGrades::builder().run(backend)?;
    println!("{grades:#?}");

    Ok(())
}
