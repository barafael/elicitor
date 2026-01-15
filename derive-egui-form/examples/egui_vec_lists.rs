//! Vec/List example
//!
//! Run with: cargo run -p derive-egui-form --example vec_lists

use derive_egui_form::EguiBackend;
use example_surveys::{ShoppingList, StudentGrades};

fn main() -> anyhow::Result<()> {
    let backend = EguiBackend::new();
    let shopping = ShoppingList::builder().run(backend)?;
    println!("{shopping:#?}");

    let backend = EguiBackend::new();
    let grades = StudentGrades::builder().run(backend)?;
    println!("{grades:#?}");

    Ok(())
}
