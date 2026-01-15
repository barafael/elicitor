//! Vec lists example - generate HTML forms with list inputs.
//!
//! Demonstrates:
//! - Vec<String> for collecting lists of strings
//! - Vec<numeric> for collecting lists of numbers
//! - Min/max bounds on numeric list elements
//!
//! Run with: cargo run -p derive-html-document --example vec_lists

use derive_html_document::to_html;
use example_surveys::{ShoppingList, StudentGrades};

fn main() {
    let html = to_html::<ShoppingList>(Some("Shopping List"));
    std::fs::write("shopping_list.html", &html).expect("Failed to write HTML file");
    println!("Generated shopping_list.html");

    let html = to_html::<StudentGrades>(Some("Student Grades"));
    std::fs::write("student_grades.html", &html).expect("Failed to write HTML file");
    println!("Generated student_grades.html");
}
