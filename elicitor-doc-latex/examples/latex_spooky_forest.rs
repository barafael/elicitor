//! Generate a fillable LaTeX form for the SpookyForest survey.

use elicitor::Survey;
use elicitor_doc_latex::to_latex_form;
use example_surveys::SpookyForest;
use std::fs::File;
use std::io::Write;

fn main() -> anyhow::Result<()> {
    // Build the survey definition from the type
    let survey = SpookyForest::survey();
    let latex = to_latex_form(&survey);
    let mut file = File::create("spooky_forest_form.tex")?;
    file.write_all(latex.as_bytes())?;
    // println!("LaTeX form written to spooky_forest_form.tex");
    Ok(())
}
