#[cfg(feature = "typst-backend")]
use crate::backend::{AnswerValue, Answers, BackendError, InterviewBackend};
use crate::interview::{Interview, Question, QuestionKind};
use std::path::PathBuf;

/// Typst-based PDF form generation backend
///
/// This backend generates a fillable PDF form using Typst markup language.
/// Unlike interactive backends, it creates a document that can be filled out manually.
pub struct TypstBackend {
    output_path: PathBuf,
    title: Option<String>,
}

impl TypstBackend {
    pub fn new(output_path: impl Into<PathBuf>) -> Self {
        Self {
            output_path: output_path.into(),
            title: None,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    fn generate_typst_markup(&self, interview: &Interview) -> String {
        let mut markup = String::new();

        // Document setup
        markup.push_str("#set page(paper: \"us-letter\", margin: 1in)\n");
        markup.push_str("#set text(font: \"New Computer Modern\", size: 11pt)\n");
        markup.push_str("#set par(justify: true)\n\n");

        // Title
        if let Some(title) = &self.title {
            markup.push_str(&format!(
                "#align(center)[#text(size: 18pt, weight: \"bold\")[{}]]\n\n",
                title
            ));
        }

        // Prelude
        if let Some(prelude) = &interview.prelude {
            markup.push_str(&format!("#text(style: \"italic\")[{}]\n\n", prelude));
            markup.push_str("#line(length: 100%)\n\n");
        }

        // Generate form fields for each question
        for question in &interview.sections {
            self.generate_question_markup(&mut markup, question, "");
        }

        // Epilogue
        if let Some(epilogue) = &interview.epilogue {
            markup.push_str("\n#line(length: 100%)\n\n");
            markup.push_str(&format!("#text(style: \"italic\")[{}]\n", epilogue));
        }

        markup
    }

    fn generate_question_markup(&self, markup: &mut String, question: &Question, prefix: &str) {
        match question.kind() {
            QuestionKind::Input(_) | QuestionKind::Multiline(_) | QuestionKind::Masked(_) => {
                markup.push_str(&format!("*{}*\n\n", question.prompt()));
                markup.push_str(
                    "#box(width: 100%, height: 1.5em, stroke: 0.5pt + gray, radius: 2pt)[\n",
                );
                markup.push_str("  #h(0.3em)\n");
                markup.push_str("]\n\n");
            }
            QuestionKind::Int(int_q) | QuestionKind::Float(float_q) => {
                let constraints = if let QuestionKind::Int(q) = question.kind() {
                    match (q.min, q.max) {
                        (Some(min), Some(max)) => format!(" (Range: {} - {})", min, max),
                        (Some(min), None) => format!(" (Min: {})", min),
                        (None, Some(max)) => format!(" (Max: {})", max),
                        _ => String::new(),
                    }
                } else if let QuestionKind::Float(q) = question.kind() {
                    match (q.min, q.max) {
                        (Some(min), Some(max)) => format!(" (Range: {} - {})", min, max),
                        (Some(min), None) => format!(" (Min: {})", min),
                        (None, Some(max)) => format!(" (Max: {})", max),
                        _ => String::new(),
                    }
                } else {
                    String::new()
                };

                markup.push_str(&format!("*{}*{}\n\n", question.prompt(), constraints));
                markup.push_str(
                    "#box(width: 100%, height: 1.5em, stroke: 0.5pt + gray, radius: 2pt)[\n",
                );
                markup.push_str("  #h(0.3em)\n");
                markup.push_str("]\n\n");
            }
            QuestionKind::Confirm(_) => {
                markup.push_str(&format!("*{}*\n\n", question.prompt()));
                markup.push_str("#grid(\n");
                markup.push_str("  columns: (auto, 1fr, auto, 1fr),\n");
                markup.push_str("  gutter: 0.5em,\n");
                markup.push_str("  box(width: 1em, height: 1em, stroke: 0.5pt + black),\n");
                markup.push_str("  [Yes],\n");
                markup.push_str("  box(width: 1em, height: 1em, stroke: 0.5pt + black),\n");
                markup.push_str("  [No]\n");
                markup.push_str(")\n\n");
            }
            QuestionKind::Sequence(questions) => {
                // Check if this is an enum alternatives sequence
                let is_enum_alternatives = !questions.is_empty()
                    && questions
                        .iter()
                        .all(|q| matches!(q.kind(), QuestionKind::Alternative(_, _)));

                if is_enum_alternatives {
                    // Enum - show as radio buttons
                    markup.push_str(&format!("*{}*\n\n", question.prompt()));

                    for variant in questions {
                        markup.push_str("#grid(\n");
                        markup.push_str("  columns: (auto, 1fr),\n");
                        markup.push_str("  gutter: 0.5em,\n");
                        markup.push_str(
                            "  box(width: 1em, height: 1em, stroke: 0.5pt + black, radius: 50%),\n",
                        );
                        markup.push_str(&format!("  [{}]\n", variant.name()));
                        markup.push_str(")\n\n");

                        // If variant has fields, show them indented
                        if let QuestionKind::Alternative(_, fields) = variant.kind() {
                            if !fields.is_empty() {
                                markup.push_str("#box(inset: (left: 2em))[\n");
                                for field in fields {
                                    self.generate_question_markup(markup, field, "");
                                }
                                markup.push_str("]\n\n");
                            }
                        }
                    }
                } else {
                    // Regular sequence
                    for q in questions {
                        self.generate_question_markup(markup, q, prefix);
                    }
                }
            }
            QuestionKind::Alternative(_, alternatives) => {
                // This shouldn't normally be reached as alternatives are wrapped in sequences
                for alt in alternatives {
                    self.generate_question_markup(markup, alt, prefix);
                }
            }
            QuestionKind::MultiSelect(multi_q) => {
                // Multi-select - show as checkboxes
                markup.push_str(&format!("*{}* _(select all that apply)_\n\n", question.prompt()));
                
                for option in &multi_q.options {
                    markup.push_str("#grid(\n");
                    markup.push_str("  columns: (auto, 1fr),\n");
                    markup.push_str("  gutter: 0.5em,\n");
                    markup.push_str("  box(width: 1em, height: 1em, stroke: 0.5pt + black),\n");
                    markup.push_str(&format!("  [{}]\n", option));
                    markup.push_str(")\n");
                }
                markup.push_str("\n");
            }
        }
    }

    fn compile_to_pdf(&self, markup: &str) -> Result<Vec<u8>, BackendError> {
        use typst::World;

        // Create a simple in-memory world for compilation
        let world = SimpleWorld::new(markup.to_string());

        // Compile the markup
        let document = typst::compile(&world).map_err(|e| {
            BackendError::ExecutionError(format!("Typst compilation failed: {:?}", e))
})?;

        // Generate PDF
        let pdf_data = typst_pdf::pdf(&document, None)
            .map_err(|e| BackendError::ExecutionError(format!("PDF generation failed: {:?}", e)))?;

        Ok(pdf_data)
    }
}

impl InterviewBackend for TypstBackend {
    fn execute(&self, interview: &Interview) -> Result<Answers, BackendError> {
        // Generate Typst markup
        let markup = self.generate_typst_markup(interview);

        // Compile to PDF
        let pdf_data = self.compile_to_pdf(&markup)?;

        // Write PDF to file
        std::fs::write(&self.output_path, pdf_data)
            .map_err(|e| BackendError::ExecutionError(format!("Failed to write PDF: {}", e)))?;

        println!("PDF form generated: {}", self.output_path.display());
        println!("Please fill out the form and provide answers manually.");

        // Return empty answers since this backend generates a form to be filled manually
        Ok(Answers::default())
    }
}

// Minimal World implementation for Typst compilation
struct SimpleWorld {
    source: String,
}

impl SimpleWorld {
    fn new(source: String) -> Self {
        Self { source }
    }
}

impl typst::World for SimpleWorld {
    fn library(&self) -> &typst::foundations::Library {
        typst::foundations::Library::default()
    }

    fn book(&self) -> &typst::foundations::Prehashed<typst::foundations::Book> {
        // Return empty book
        todo!("Implement book() for SimpleWorld")
    }

    fn main(&self) -> &typst::foundations::Source {
        // Return main source
        todo!("Implement main() for SimpleWorld")
    }

    fn resolve(
        &self,
        _spec: &typst::foundations::FileSpec,
    ) -> Result<typst::foundations::FileId, typst::diag::FileError> {
        todo!("Implement resolve() for SimpleWorld")
    }

    fn source(&self, _id: typst::foundations::FileId) -> &typst::foundations::Source {
        todo!("Implement source() for SimpleWorld")
    }

    fn file(
        &self,
        _id: typst::foundations::FileId,
    ) -> Result<typst::foundations::Bytes, typst::diag::FileError> {
        todo!("Implement file() for SimpleWorld")
    }

    fn font(&self, _index: usize) -> Option<typst::text::Font> {
        todo!("Implement font() for SimpleWorld")
    }

    fn today(&self, _offset: Option<i64>) -> Option<typst::foundations::Datetime> {
        Some(typst::foundations::Datetime::from_ymd(2025, 1, 2))
    }
}
