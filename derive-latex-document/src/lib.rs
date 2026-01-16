//! LaTeX backend for derive-survey: generates fillable PDF forms from SurveyDefinition.

use derive_survey::SurveyDefinition;

/// Escape special LaTeX characters in text content.
fn escape_latex(s: &str) -> String {
    s.replace('\\', "\\textbackslash{}")
        .replace('&', "\\&")
        .replace('%', "\\%")
        .replace('$', "\\$")
        .replace('#', "\\#")
        .replace('_', "\\_")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('~', "\\textasciitilde{}")
        .replace('^', "\\textasciicircum{}")
}

/// Sanitize a field name for use in PDF form field names.
/// PDF field names should not contain special characters.
fn sanitize_field_name(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => c,
            _ => '-',
        })
        .collect()
}

/// Calculate shade percentage based on nesting depth.
/// Starts at 5% and increases by 5% per level, capped at 25%.
fn shade_percent(indent_level: usize) -> usize {
    (5 + indent_level * 5).min(25)
}

/// Generate a LaTeX document (as a String) for a fillable form from a SurveyDefinition.
pub fn to_latex_form(survey: &SurveyDefinition) -> String {
    let mut latex = String::new();

    // Document preamble
    latex.push_str(
        r#"\documentclass[11pt]{article}
\usepackage[utf8]{inputenc}
\usepackage[T1]{fontenc}
\usepackage[sfdefault]{cabin}
\usepackage[pdftex]{hyperref}
\usepackage{geometry}
\usepackage{xcolor}
\usepackage{tcolorbox}

\geometry{margin=1in}
\hypersetup{
    colorlinks=true,
    linkcolor=blue,
    pdfborder={0 0 0}
}

% Force consistent checkbox appearance
\renewcommand{\LayoutCheckField}[2]{\makebox[12pt][l]{#2}}

% Shaded blocks for nested content with varying depth and rounded corners
\newtcolorbox{shadedblock}[1][5]{
    colback=black!#1,
    colframe=black!#1,
    arc=3pt,
    boxrule=0pt,
    left=0.3em,
    right=0.3em,
    top=0.3em,
    bottom=0.3em,
    boxsep=0pt
}



\begin{document}
"#,
    );

    // Prelude
    if let Some(prelude) = &survey.prelude {
        latex.push_str("\n\\noindent ");
        latex.push_str(&escape_latex(prelude));
        latex.push_str("\n\n\\vspace{1em}\n");
    }

    latex.push_str("\n\\begin{Form}\n");

    for (i, q) in survey.questions.iter().enumerate() {
        if i > 0 {
            latex.push_str("\n\\vspace{1.5em}\n");
        }
        latex.push_str(&render_question(q, 0));
    }

    latex.push_str("\n\\end{Form}\n");

    // Epilogue
    if let Some(epilogue) = &survey.epilogue {
        latex.push_str("\n\\vspace{2em}\n\\noindent ");
        latex.push_str(&escape_latex(epilogue));
        latex.push_str("\n");
    }

    latex.push_str("\n\\end{document}\n");
    latex
}

fn render_question(q: &derive_survey::Question, indent_level: usize) -> String {
    render_question_with_path(q, indent_level, None)
}

fn render_question_with_path(
    q: &derive_survey::Question,
    indent_level: usize,
    parent_path: Option<&str>,
) -> String {
    use derive_survey::QuestionKind;

    let mut s = String::new();
    let indent = "  ".repeat(indent_level);
    let ask = q.ask();

    // Build the full path - combine parent path with question's path
    let path_str = q.path().as_str();
    let full_path = match (parent_path, path_str.is_empty()) {
        (Some(parent), true) => parent.to_string(),
        (Some(parent), false) => format!("{}.{}", parent, path_str),
        (None, _) => path_str.to_string(),
    };
    let field_name = sanitize_field_name(&full_path);

    // Render the question text if present
    if !ask.is_empty() {
        s.push_str(&indent);
        s.push_str("\\noindent\\textbf{");
        s.push_str(&escape_latex(ask));
        s.push_str("}\n\n");
        s.push_str(&indent);
        s.push_str("\\smallskip\n");
    }

    match q.kind() {
        QuestionKind::Input(_) => {
            s.push_str(&indent);
            s.push_str("\\noindent\\TextField[name=");
            s.push_str(&field_name);
            s.push_str(",width=4in,bordercolor={0.5 0.5 0.5}]{}\n");
            s.push_str(&indent);
            s.push_str("\\par\\medskip\n");
        }
        QuestionKind::Int(int_q) => {
            s.push_str(&indent);
            s.push_str("\\noindent\\TextField[name=");
            s.push_str(&field_name);
            s.push_str(",width=1.5in,bordercolor={0.5 0.5 0.5}]{}");

            // Add range hint if available
            if int_q.min.is_some() || int_q.max.is_some() {
                s.push_str(" \\textit{\\small(");
                match (int_q.min, int_q.max) {
                    (Some(min), Some(max)) => s.push_str(&format!("{} -- {}", min, max)),
                    (Some(min), None) => s.push_str(&format!("min: {}", min)),
                    (None, Some(max)) => s.push_str(&format!("max: {}", max)),
                    (None, None) => {}
                }
                s.push_str(")}");
            }
            s.push_str("\n");
            s.push_str(&indent);
            s.push_str("\\par\\medskip\n");
        }
        QuestionKind::Float(float_q) => {
            s.push_str(&indent);
            s.push_str("\\noindent\\TextField[name=");
            s.push_str(&field_name);
            s.push_str(",width=1.5in,bordercolor={0.5 0.5 0.5}]{}");

            // Add range hint if available
            if float_q.min.is_some() || float_q.max.is_some() {
                s.push_str(" \\textit{\\small(");
                match (float_q.min, float_q.max) {
                    (Some(min), Some(max)) => s.push_str(&format!("{} -- {}", min, max)),
                    (Some(min), None) => s.push_str(&format!("min: {}", min)),
                    (None, Some(max)) => s.push_str(&format!("max: {}", max)),
                    (None, None) => {}
                }
                s.push_str(")}");
            }
            s.push_str("\n");
            s.push_str(&indent);
            s.push_str("\\par\\medskip\n");
        }
        QuestionKind::Confirm(_) => {
            s.push_str(&indent);
            s.push_str("\\noindent\\CheckBox[name=");
            s.push_str(&field_name);
            s.push_str(
                ",width=10pt,height=10pt,borderwidth=1pt,bordercolor={0.4 0.4 0.4}]{} Yes\n\n",
            );
        }
        QuestionKind::OneOf(oneof) => {
            s.push_str(&indent);
            s.push_str("\\noindent\\ChoiceMenu[combo,name=");
            s.push_str(&field_name);
            s.push_str(",width=3in,bordercolor={0.5 0.5 0.5}]{}{");
            let options: Vec<String> = oneof
                .variants
                .iter()
                .map(|v| escape_latex(&v.name))
                .collect();
            s.push_str(&options.join(","));
            s.push_str("}\n");

            // Render follow-up fields for variants that have nested questions
            for variant in &oneof.variants {
                if !matches!(variant.kind, derive_survey::QuestionKind::Unit) {
                    s.push_str("\n");
                    s.push_str(&indent);
                    s.push_str("\\vspace{0.5em}\n");
                    s.push_str(&indent);
                    s.push_str("\\textit{If ");
                    s.push_str(&escape_latex(&variant.name));
                    s.push_str(":}\n\n");
                    s.push_str(&indent);
                    s.push_str(&format!(
                        "\\begin{{shadedblock}}[{}]\n",
                        shade_percent(indent_level + 1)
                    ));
                    s.push_str(&render_variant_fields(
                        &variant.kind,
                        &full_path,
                        indent_level + 1,
                    ));
                    s.push_str(&indent);
                    s.push_str("\\end{shadedblock}\n");
                }
            }
        }
        QuestionKind::AnyOf(anyof) => {
            for variant in &anyof.variants {
                let checkbox_name =
                    format!("{}-{}", field_name, sanitize_field_name(&variant.name));
                s.push_str(&indent);
                s.push_str("\\CheckBox[name=");
                s.push_str(&checkbox_name);
                s.push_str(",width=10pt,height=10pt,borderwidth=1pt,bordercolor={0.4 0.4 0.4}]{} ");
                s.push_str(&escape_latex(&variant.name));
                s.push_str("\n\n");
                s.push_str(&indent);
                s.push_str("\\vspace{0.3em}\n");
            }

            // Render follow-up fields for variants that have nested questions
            for variant in &anyof.variants {
                if !matches!(variant.kind, derive_survey::QuestionKind::Unit) {
                    s.push_str("\n");
                    s.push_str(&indent);
                    s.push_str("\\vspace{0.5em}\n");
                    s.push_str(&indent);
                    s.push_str("\\textit{If ");
                    s.push_str(&escape_latex(&variant.name));
                    s.push_str(":}\n\n");
                    s.push_str(&indent);
                    s.push_str(&format!(
                        "\\begin{{shadedblock}}[{}]\n",
                        shade_percent(indent_level + 1)
                    ));
                    s.push_str(&render_variant_fields(
                        &variant.kind,
                        &full_path,
                        indent_level + 1,
                    ));
                    s.push_str(&indent);
                    s.push_str("\\end{shadedblock}\n");
                }
            }
        }
        QuestionKind::AllOf(allof) => {
            // Nested questions - render with increased indent, passing current path as parent
            let parent = if full_path.is_empty() {
                None
            } else {
                Some(full_path.as_str())
            };
            s.push_str(&indent);
            s.push_str(&format!(
                "\\begin{{shadedblock}}[{}]\n",
                shade_percent(indent_level + 1)
            ));
            for (i, sub) in allof.questions.iter().enumerate() {
                if i > 0 {
                    s.push_str("\n");
                    s.push_str(&indent);
                    s.push_str("\\vspace{0.8em}\n");
                }
                s.push_str(&render_question_with_path(sub, indent_level + 1, parent));
            }
            s.push_str(&indent);
            s.push_str("\\end{shadedblock}\n");
        }
        QuestionKind::Multiline(_) => {
            s.push_str(&indent);
            s.push_str("\\noindent\\TextField[name=");
            s.push_str(&field_name);
            s.push_str(",multiline=true,width=4in,height=1.2in,bordercolor={0.5 0.5 0.5}]{}\n\n");
        }
        QuestionKind::Unit => {
            // No input needed for unit types
        }
        QuestionKind::Masked(_) => {
            s.push_str(&indent);
            s.push_str("\\noindent\\TextField[name=");
            s.push_str(&field_name);
            s.push_str(",password=true,width=3in,bordercolor={0.5 0.5 0.5}]{}\n\n");
        }
        QuestionKind::List(_) => {
            s.push_str(&indent);
            s.push_str("\\noindent\\TextField[name=");
            s.push_str(&field_name);
            s.push_str(
                ",width=4in,bordercolor={0.5 0.5 0.5}]{} \\textit{\\small(comma-separated)}\n\n",
            );
        }
    }

    s
}

/// Render the fields for a variant's nested QuestionKind
fn render_variant_fields(
    kind: &derive_survey::QuestionKind,
    parent_path: &str,
    indent_level: usize,
) -> String {
    use derive_survey::QuestionKind;

    let indent = "  ".repeat(indent_level);
    let mut s = String::new();

    match kind {
        QuestionKind::Unit => {
            // No fields for unit variants
        }
        QuestionKind::Input(input_q) => {
            let field_name = sanitize_field_name(parent_path);
            s.push_str(&indent);
            s.push_str("\\noindent\\TextField[name=");
            s.push_str(&field_name);
            s.push_str("-value,width=4in,bordercolor={0.5 0.5 0.5}]{}");
            if let Some(default) = &input_q.default {
                s.push_str(" \\textit{\\small(default: ");
                s.push_str(&escape_latex(default));
                s.push_str(")}");
            }
            s.push_str("\n\n");
        }
        QuestionKind::Int(int_q) => {
            let field_name = sanitize_field_name(parent_path);
            s.push_str(&indent);
            s.push_str("\\noindent\\TextField[name=");
            s.push_str(&field_name);
            s.push_str("-value,width=1.5in,bordercolor={0.5 0.5 0.5}]{}");
            if int_q.min.is_some() || int_q.max.is_some() {
                s.push_str(" \\textit{\\small(");
                match (int_q.min, int_q.max) {
                    (Some(min), Some(max)) => s.push_str(&format!("{} -- {}", min, max)),
                    (Some(min), None) => s.push_str(&format!("min: {}", min)),
                    (None, Some(max)) => s.push_str(&format!("max: {}", max)),
                    (None, None) => {}
                }
                s.push_str(")}");
            }
            s.push_str("\n\n");
        }
        QuestionKind::Float(float_q) => {
            let field_name = sanitize_field_name(parent_path);
            s.push_str(&indent);
            s.push_str("\\noindent\\TextField[name=");
            s.push_str(&field_name);
            s.push_str("-value,width=1.5in,bordercolor={0.5 0.5 0.5}]{}");
            if float_q.min.is_some() || float_q.max.is_some() {
                s.push_str(" \\textit{\\small(");
                match (float_q.min, float_q.max) {
                    (Some(min), Some(max)) => s.push_str(&format!("{} -- {}", min, max)),
                    (Some(min), None) => s.push_str(&format!("min: {}", min)),
                    (None, Some(max)) => s.push_str(&format!("max: {}", max)),
                    (None, None) => {}
                }
                s.push_str(")}");
            }
            s.push_str("\n\n");
        }
        QuestionKind::Confirm(_) => {
            let field_name = sanitize_field_name(parent_path);
            s.push_str(&indent);
            s.push_str("\\noindent\\CheckBox[name=");
            s.push_str(&field_name);
            s.push_str(
                "-value,width=10pt,height=10pt,borderwidth=1pt,bordercolor={0.4 0.4 0.4}]{} Yes\n\n",
            );
        }
        QuestionKind::Multiline(_) => {
            let field_name = sanitize_field_name(parent_path);
            s.push_str(&indent);
            s.push_str("\\noindent\\TextField[name=");
            s.push_str(&field_name);
            s.push_str(
                "-value,multiline=true,width=4in,height=1.2in,bordercolor={0.5 0.5 0.5}]{}\n\n",
            );
        }
        QuestionKind::AllOf(allof) => {
            // Struct variant - render all nested questions
            for (i, sub) in allof.questions.iter().enumerate() {
                if i > 0 {
                    s.push_str(&indent);
                    s.push_str("\\vspace{0.5em}\n");
                }
                s.push_str(&render_question_with_path(
                    sub,
                    indent_level,
                    Some(parent_path),
                ));
            }
        }
        QuestionKind::OneOf(oneof) => {
            // Nested enum - render as choice menu with its own follow-ups
            let field_name = sanitize_field_name(parent_path);
            s.push_str(&indent);
            s.push_str("\\noindent\\ChoiceMenu[combo,name=");
            s.push_str(&field_name);
            s.push_str("-value,width=3in,bordercolor={0.5 0.5 0.5}]{}{");
            let options: Vec<String> = oneof
                .variants
                .iter()
                .map(|v| escape_latex(&v.name))
                .collect();
            s.push_str(&options.join(","));
            s.push_str("}\n");

            // Recursively render nested variant fields
            for variant in &oneof.variants {
                if !matches!(variant.kind, QuestionKind::Unit) {
                    s.push_str("\n");
                    s.push_str(&indent);
                    s.push_str("\\vspace{0.3em}\n");
                    s.push_str(&indent);
                    s.push_str("\\textit{\\small If ");
                    s.push_str(&escape_latex(&variant.name));
                    s.push_str(":}\n\n");
                    s.push_str(&indent);
                    s.push_str(&format!(
                        "\\begin{{shadedblock}}[{}]\n",
                        shade_percent(indent_level + 1)
                    ));
                    let nested_path =
                        format!("{}-{}", parent_path, sanitize_field_name(&variant.name));
                    s.push_str(&render_variant_fields(
                        &variant.kind,
                        &nested_path,
                        indent_level + 1,
                    ));
                    s.push_str(&indent);
                    s.push_str("\\end{shadedblock}\n");
                }
            }
        }
        QuestionKind::AnyOf(anyof) => {
            // Multi-select within a variant
            for variant in &anyof.variants {
                let checkbox_name = format!(
                    "{}-{}",
                    sanitize_field_name(parent_path),
                    sanitize_field_name(&variant.name)
                );
                s.push_str(&indent);
                s.push_str("\\CheckBox[name=");
                s.push_str(&checkbox_name);
                s.push_str(",width=10pt,height=10pt,borderwidth=1pt,bordercolor={0.4 0.4 0.4}]{} ");
                s.push_str(&escape_latex(&variant.name));
                s.push_str("\n\n");
                s.push_str(&indent);
                s.push_str("\\vspace{0.3em}\n");
            }
        }
        QuestionKind::Masked(_) => {
            let field_name = sanitize_field_name(parent_path);
            s.push_str(&indent);
            s.push_str("\\noindent\\TextField[name=");
            s.push_str(&field_name);
            s.push_str("-value,password=true,width=3in,bordercolor={0.5 0.5 0.5}]{}\n");
        }
        QuestionKind::List(_) => {
            let field_name = sanitize_field_name(parent_path);
            s.push_str(&indent);
            s.push_str("\\noindent\\TextField[name=");
            s.push_str(&field_name);
            s.push_str("-value,width=4in,bordercolor={0.5 0.5 0.5}]{} \\textit{\\small(comma-separated)}\n");
        }
    }

    s
}
