//! HTML form generator implementation.

use derive_survey::{
    DefaultValue, ListElementKind, Question, QuestionKind, Survey, SurveyDefinition,
};

/// Options for HTML generation.
#[derive(Debug, Clone, Default)]
pub struct HtmlOptions {
    /// Title for the HTML document.
    pub title: Option<String>,
    /// Whether to include default CSS styling.
    pub include_styles: bool,
    /// Whether to generate a complete HTML document (with html/head/body tags).
    pub full_document: bool,
    /// Custom CSS class prefix for all generated elements.
    pub class_prefix: String,
}

impl HtmlOptions {
    /// Create new options with default values.
    pub fn new() -> Self {
        Self {
            title: None,
            include_styles: true,
            full_document: true,
            class_prefix: "survey".to_string(),
        }
    }

    /// Set the document title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Enable or disable default CSS styling.
    pub fn with_styles(mut self, include: bool) -> Self {
        self.include_styles = include;
        self
    }

    /// Generate a complete HTML document or just the form fragment.
    pub fn full_document(mut self, full: bool) -> Self {
        self.full_document = full;
        self
    }

    /// Set a custom CSS class prefix.
    pub fn with_class_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.class_prefix = prefix.into();
        self
    }
}

/// Generate an HTML form from a survey type.
///
/// This is a convenience function that uses default options with the given title.
pub fn to_html<T: Survey>(title: Option<&str>) -> String {
    let mut options = HtmlOptions::new();
    if let Some(t) = title {
        options.title = Some(t.to_string());
    }
    to_html_with_options::<T>(options)
}

/// Generate an HTML form with custom options.
pub fn to_html_with_options<T: Survey>(options: HtmlOptions) -> String {
    let definition = T::survey();
    generate_html(&definition, &options)
}

/// Generate HTML from a survey definition.
fn generate_html(definition: &SurveyDefinition, options: &HtmlOptions) -> String {
    let mut html = String::new();
    let prefix = &options.class_prefix;

    if options.full_document {
        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("  <meta charset=\"UTF-8\">\n");
        html.push_str(
            "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );

        if let Some(title) = &options.title {
            html.push_str(&format!("  <title>{}</title>\n", escape_html(title)));
        }

        if options.include_styles {
            html.push_str(&generate_styles(prefix));
        }

        html.push_str("</head>\n<body>\n");
    }

    html.push_str(&format!("<form class=\"{prefix}-form\">\n"));

    // Prelude
    if let Some(prelude) = &definition.prelude {
        html.push_str(&format!(
            "  <div class=\"{prefix}-prelude\">{}</div>\n",
            escape_html(prelude)
        ));
    }

    // Title
    if let Some(title) = &options.title {
        html.push_str(&format!(
            "  <h1 class=\"{prefix}-title\">{}</h1>\n",
            escape_html(title)
        ));
    }

    // Questions
    html.push_str(&format!("  <div class=\"{prefix}-questions\">\n"));
    for question in definition.questions() {
        html.push_str(&generate_question(question, prefix, 2, None));
    }
    html.push_str("  </div>\n");

    // Epilogue
    if let Some(epilogue) = &definition.epilogue {
        html.push_str(&format!(
            "  <div class=\"{prefix}-epilogue\">{}</div>\n",
            escape_html(epilogue)
        ));
    }

    // Submit button
    html.push_str(&format!(
        "  <button type=\"submit\" class=\"{prefix}-submit\">Submit</button>\n"
    ));

    html.push_str("</form>\n");

    if options.full_document {
        html.push_str("</body>\n</html>\n");
    }

    html
}

/// Generate HTML for a single question.
fn generate_question(
    question: &Question,
    prefix: &str,
    indent: usize,
    parent_path: Option<&str>,
) -> String {
    let ind = "  ".repeat(indent);

    // Build the full path
    let question_path = question.path().as_str();
    let path = match (parent_path, question_path.is_empty()) {
        (Some(parent), true) => parent.to_string(),
        (Some(parent), false) => format!("{parent}.{question_path}"),
        (None, _) => question_path.to_string(),
    };

    let label = format_label(question.ask(), &path);
    let field_id = path.replace('.', "-");

    let mut html = String::new();

    // Get default value info
    let (default_value, is_assumed) = match question.default() {
        DefaultValue::Suggested(v) => (Some(v), false),
        DefaultValue::Assumed(v) => (Some(v), true),
        DefaultValue::None => (None, false),
    };

    // Skip assumed fields entirely (they won't be shown in the form)
    if is_assumed {
        return html;
    }

    match question.kind() {
        QuestionKind::Unit => {
            // Unit types don't need input fields
        }

        QuestionKind::Input(_) => {
            let value_attr = default_value
                .and_then(|v| v.as_str())
                .map(|s| format!(" value=\"{}\"", escape_html(s)))
                .unwrap_or_default();

            html.push_str(&format!("{ind}<div class=\"{prefix}-field\">\n"));
            html.push_str(&format!(
                "{ind}  <label for=\"{field_id}\">{}</label>\n",
                escape_html(&label)
            ));
            html.push_str(&format!(
                "{ind}  <input type=\"text\" id=\"{field_id}\" name=\"{path}\" class=\"{prefix}-input\"{value_attr}>\n"
            ));
            html.push_str(&format!("{ind}</div>\n"));
        }

        QuestionKind::Multiline(_) => {
            let content = default_value
                .and_then(|v| v.as_str())
                .map(|s| escape_html(s))
                .unwrap_or_default();

            html.push_str(&format!("{ind}<div class=\"{prefix}-field\">\n"));
            html.push_str(&format!(
                "{ind}  <label for=\"{field_id}\">{}</label>\n",
                escape_html(&label)
            ));
            html.push_str(&format!(
                "{ind}  <textarea id=\"{field_id}\" name=\"{path}\" rows=\"4\" class=\"{prefix}-textarea\">{content}</textarea>\n"
            ));
            html.push_str(&format!("{ind}</div>\n"));
        }

        QuestionKind::Masked(_) => {
            // Don't pre-fill password fields for security
            html.push_str(&format!("{ind}<div class=\"{prefix}-field\">\n"));
            html.push_str(&format!(
                "{ind}  <label for=\"{field_id}\">{}</label>\n",
                escape_html(&label)
            ));
            html.push_str(&format!(
                "{ind}  <input type=\"password\" id=\"{field_id}\" name=\"{path}\" class=\"{prefix}-input\">\n"
            ));
            html.push_str(&format!("{ind}</div>\n"));
        }

        QuestionKind::Int(int_q) => {
            let value_attr = default_value
                .and_then(|v| v.as_int())
                .map(|i| format!(" value=\"{i}\""))
                .unwrap_or_default();

            html.push_str(&format!("{ind}<div class=\"{prefix}-field\">\n"));
            html.push_str(&format!(
                "{ind}  <label for=\"{field_id}\">{}</label>\n",
                escape_html(&label)
            ));

            let mut attrs = format!(
                "type=\"number\" id=\"{field_id}\" name=\"{path}\" class=\"{prefix}-input\""
            );
            if let Some(min) = int_q.min {
                attrs.push_str(&format!(" min=\"{min}\""));
            }
            if let Some(max) = int_q.max {
                attrs.push_str(&format!(" max=\"{max}\""));
            }

            html.push_str(&format!("{ind}  <input {attrs}{value_attr}>\n"));
            html.push_str(&format!("{ind}</div>\n"));
        }

        QuestionKind::Float(float_q) => {
            let value_attr = default_value
                .and_then(|v| v.as_float())
                .map(|f| format!(" value=\"{f}\""))
                .unwrap_or_default();

            html.push_str(&format!("{ind}<div class=\"{prefix}-field\">\n"));
            html.push_str(&format!(
                "{ind}  <label for=\"{field_id}\">{}</label>\n",
                escape_html(&label)
            ));

            let mut attrs = format!(
                "type=\"number\" step=\"any\" id=\"{field_id}\" name=\"{path}\" class=\"{prefix}-input\""
            );
            if let Some(min) = float_q.min {
                attrs.push_str(&format!(" min=\"{min}\""));
            }
            if let Some(max) = float_q.max {
                attrs.push_str(&format!(" max=\"{max}\""));
            }

            html.push_str(&format!("{ind}  <input {attrs}{value_attr}>\n"));
            html.push_str(&format!("{ind}</div>\n"));
        }

        QuestionKind::Confirm(confirm_q) => {
            // Use suggested value if provided, otherwise fall back to confirm_q.default
            let is_checked = default_value
                .and_then(|v| v.as_bool())
                .unwrap_or(confirm_q.default);
            let checked = if is_checked { " checked" } else { "" };

            html.push_str(&format!(
                "{ind}<div class=\"{prefix}-field {prefix}-checkbox\">\n"
            ));
            html.push_str(&format!(
                "{ind}  <input type=\"checkbox\" id=\"{field_id}\" name=\"{path}\"{checked}>\n"
            ));
            html.push_str(&format!(
                "{ind}  <label for=\"{field_id}\">{}</label>\n",
                escape_html(&label)
            ));
            html.push_str(&format!("{ind}</div>\n"));
        }

        QuestionKind::List(list_q) => {
            let type_hint = match &list_q.element_kind {
                ListElementKind::String => "comma-separated text",
                ListElementKind::Int { .. } => "comma-separated integers",
                ListElementKind::Float { .. } => "comma-separated numbers",
            };

            html.push_str(&format!("{ind}<div class=\"{prefix}-field\">\n"));
            html.push_str(&format!(
                "{ind}  <label for=\"{field_id}\">{} ({})</label>\n",
                escape_html(&label),
                type_hint
            ));
            html.push_str(&format!(
                "{ind}  <input type=\"text\" id=\"{field_id}\" name=\"{path}\" class=\"{prefix}-input\" placeholder=\"value1, value2, ...\">\n"
            ));
            html.push_str(&format!("{ind}</div>\n"));
        }

        QuestionKind::OneOf(one_of) => {
            // Get default selected variant index
            let default_selected = default_value
                .and_then(|v| v.as_chosen_variant())
                .or(one_of.default);

            html.push_str(&format!(
                "{ind}<fieldset class=\"{prefix}-fieldset {prefix}-oneof\">\n"
            ));
            html.push_str(&format!(
                "{ind}  <legend>{}</legend>\n",
                escape_html(&label)
            ));

            for (idx, variant) in one_of.variants.iter().enumerate() {
                let variant_id = format!("{field_id}-{}", variant.name);
                let variant_label = if variant.name == variant.name.to_uppercase() {
                    variant.name.clone()
                } else {
                    // Convert snake_case to Title Case
                    variant
                        .name
                        .split('_')
                        .map(|word| {
                            let mut chars = word.chars();
                            match chars.next() {
                                None => String::new(),
                                Some(first) => first.to_uppercase().chain(chars).collect(),
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                };

                let checked = if default_selected == Some(idx) {
                    " checked"
                } else {
                    ""
                };

                html.push_str(&format!("{ind}  <div class=\"{prefix}-radio-option\">\n"));
                html.push_str(&format!(
                    "{ind}    <input type=\"radio\" id=\"{variant_id}\" name=\"{path}\" value=\"{idx}\"{checked}>\n"
                ));
                html.push_str(&format!(
                    "{ind}    <label for=\"{variant_id}\">{}</label>\n",
                    escape_html(&variant_label)
                ));

                // Nested fields for this variant
                if !matches!(variant.kind, QuestionKind::Unit) {
                    html.push_str(&format!(
                        "{ind}    <div class=\"{prefix}-nested\" data-variant=\"{idx}\">\n"
                    ));
                    html.push_str(&generate_variant_fields(
                        &variant.kind,
                        &format!("{path}.{}", variant.name),
                        prefix,
                        indent + 3,
                    ));
                    html.push_str(&format!("{ind}    </div>\n"));
                }

                html.push_str(&format!("{ind}  </div>\n"));
            }

            html.push_str(&format!("{ind}</fieldset>\n"));
        }

        QuestionKind::AnyOf(any_of) => {
            // Get default selected variant indices
            let default_indices: Vec<usize> = default_value
                .and_then(|v| v.as_chosen_variants())
                .map(|v| v.to_vec())
                .unwrap_or_else(|| any_of.defaults.clone());

            html.push_str(&format!(
                "{ind}<fieldset class=\"{prefix}-fieldset {prefix}-anyof\">\n"
            ));
            html.push_str(&format!(
                "{ind}  <legend>{}</legend>\n",
                escape_html(&label)
            ));

            for (idx, variant) in any_of.variants.iter().enumerate() {
                let variant_id = format!("{field_id}-{idx}");
                let variant_label = if variant.name == variant.name.to_uppercase() {
                    variant.name.clone()
                } else {
                    variant
                        .name
                        .split('_')
                        .map(|word| {
                            let mut chars = word.chars();
                            match chars.next() {
                                None => String::new(),
                                Some(first) => first.to_uppercase().chain(chars).collect(),
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                };

                let checked = if default_indices.contains(&idx) {
                    " checked"
                } else {
                    ""
                };

                html.push_str(&format!(
                    "{ind}  <div class=\"{prefix}-checkbox-option\">\n"
                ));
                html.push_str(&format!(
                    "{ind}    <input type=\"checkbox\" id=\"{variant_id}\" name=\"{path}[]\" value=\"{idx}\"{checked}>\n"
                ));
                html.push_str(&format!(
                    "{ind}    <label for=\"{variant_id}\">{}</label>\n",
                    escape_html(&variant_label)
                ));

                // Nested fields for this variant
                if !matches!(variant.kind, QuestionKind::Unit) {
                    html.push_str(&format!(
                        "{ind}    <div class=\"{prefix}-nested\" data-variant=\"{idx}\">\n"
                    ));
                    html.push_str(&generate_variant_fields(
                        &variant.kind,
                        &format!("{path}.{idx}"),
                        prefix,
                        indent + 3,
                    ));
                    html.push_str(&format!("{ind}    </div>\n"));
                }

                html.push_str(&format!("{ind}  </div>\n"));
            }

            html.push_str(&format!("{ind}</fieldset>\n"));
        }

        QuestionKind::AllOf(all_of) => {
            html.push_str(&format!(
                "{ind}<fieldset class=\"{prefix}-fieldset {prefix}-group\">\n"
            ));
            html.push_str(&format!(
                "{ind}  <legend>{}</legend>\n",
                escape_html(&label)
            ));

            for nested_q in all_of.questions() {
                html.push_str(&generate_question(
                    nested_q,
                    prefix,
                    indent + 1,
                    Some(&path),
                ));
            }

            html.push_str(&format!("{ind}</fieldset>\n"));
        }
    }

    html
}

/// Generate HTML for nested variant fields.
fn generate_variant_fields(
    kind: &QuestionKind,
    base_path: &str,
    prefix: &str,
    indent: usize,
) -> String {
    let ind = "  ".repeat(indent);
    let mut html = String::new();

    match kind {
        QuestionKind::Input(_) => {
            let field_id = base_path.replace('.', "-");
            html.push_str(&format!(
                "{ind}<input type=\"text\" id=\"{field_id}\" name=\"{base_path}\" class=\"{prefix}-input\" placeholder=\"Enter value...\">\n"
            ));
        }
        QuestionKind::Int(int_q) => {
            let field_id = base_path.replace('.', "-");
            let mut attrs = format!(
                "type=\"number\" id=\"{field_id}\" name=\"{base_path}\" class=\"{prefix}-input\""
            );
            if let Some(min) = int_q.min {
                attrs.push_str(&format!(" min=\"{min}\""));
            }
            if let Some(max) = int_q.max {
                attrs.push_str(&format!(" max=\"{max}\""));
            }
            html.push_str(&format!("{ind}<input {attrs}>\n"));
        }
        QuestionKind::Float(float_q) => {
            let field_id = base_path.replace('.', "-");
            let mut attrs = format!(
                "type=\"number\" step=\"any\" id=\"{field_id}\" name=\"{base_path}\" class=\"{prefix}-input\""
            );
            if let Some(min) = float_q.min {
                attrs.push_str(&format!(" min=\"{min}\""));
            }
            if let Some(max) = float_q.max {
                attrs.push_str(&format!(" max=\"{max}\""));
            }
            html.push_str(&format!("{ind}<input {attrs}>\n"));
        }
        QuestionKind::AllOf(all_of) => {
            for nested_q in all_of.questions() {
                let nested_path = format!("{}.{}", base_path, nested_q.path().as_str());
                let label = format_label(nested_q.ask(), nested_q.path().as_str());
                let field_id = nested_path.replace('.', "-");

                html.push_str(&format!("{ind}<div class=\"{prefix}-field\">\n"));
                html.push_str(&format!(
                    "{ind}  <label for=\"{field_id}\">{}</label>\n",
                    escape_html(&label)
                ));

                match nested_q.kind() {
                    QuestionKind::Input(_) | QuestionKind::Multiline(_) => {
                        html.push_str(&format!(
                            "{ind}  <input type=\"text\" id=\"{field_id}\" name=\"{nested_path}\" class=\"{prefix}-input\">\n"
                        ));
                    }
                    QuestionKind::Int(int_q) => {
                        let mut attrs = format!(
                            "type=\"number\" id=\"{field_id}\" name=\"{nested_path}\" class=\"{prefix}-input\""
                        );
                        if let Some(min) = int_q.min {
                            attrs.push_str(&format!(" min=\"{min}\""));
                        }
                        if let Some(max) = int_q.max {
                            attrs.push_str(&format!(" max=\"{max}\""));
                        }
                        html.push_str(&format!("{ind}  <input {attrs}>\n"));
                    }
                    QuestionKind::Float(float_q) => {
                        let mut attrs = format!(
                            "type=\"number\" step=\"any\" id=\"{field_id}\" name=\"{nested_path}\" class=\"{prefix}-input\""
                        );
                        if let Some(min) = float_q.min {
                            attrs.push_str(&format!(" min=\"{min}\""));
                        }
                        if let Some(max) = float_q.max {
                            attrs.push_str(&format!(" max=\"{max}\""));
                        }
                        html.push_str(&format!("{ind}  <input {attrs}>\n"));
                    }
                    _ => {
                        html.push_str(&format!(
                            "{ind}  <input type=\"text\" id=\"{field_id}\" name=\"{nested_path}\" class=\"{prefix}-input\">\n"
                        ));
                    }
                }

                html.push_str(&format!("{ind}</div>\n"));
            }
        }
        _ => {}
    }

    html
}

/// Format a prompt as a label.
fn format_label(ask: &str, path: &str) -> String {
    if ask.is_empty() {
        // Create a readable label from the path
        path.split('.')
            .last()
            .unwrap_or("")
            .split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        ask.to_string()
    }
}

/// Escape HTML special characters.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Generate default CSS styles.
fn generate_styles(prefix: &str) -> String {
    format!(
        r#"  <style>
    .{prefix}-form {{
      max-width: 600px;
      margin: 2rem auto;
      padding: 1rem;
      font-family: sans-serif;
    }}
    .{prefix}-prelude, .{prefix}-epilogue {{
      margin: 1rem 0;
      padding: 0.5rem;
      background: #f5f5f5;
      white-space: pre-wrap;
    }}
    .{prefix}-field {{
      margin: 0.5rem 0;
    }}
    .{prefix}-field label {{
      display: block;
      margin-bottom: 0.25rem;
    }}
    .{prefix}-input, .{prefix}-textarea {{
      width: 100%;
      padding: 0.5rem;
      box-sizing: border-box;
    }}
    .{prefix}-checkbox {{
      display: flex;
      align-items: center;
      gap: 0.5rem;
    }}
    .{prefix}-checkbox label {{
      display: inline;
    }}
    .{prefix}-fieldset {{
      margin: 1rem 0;
      padding: 1rem;
    }}
    .{prefix}-radio-option, .{prefix}-checkbox-option {{
      margin: 0.25rem 0;
    }}
    .{prefix}-nested {{
      margin-left: 1.5rem;
      padding-left: 0.5rem;
      border-left: 2px solid #ccc;
    }}
    .{prefix}-submit {{
      margin-top: 1rem;
      padding: 0.5rem 1rem;
    }}
  </style>
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_options_creation() {
        let _options = HtmlOptions::new();
        let _with_title = HtmlOptions::new().with_title("Test");
        let _with_styles = HtmlOptions::new().with_styles(true);
        let _full_doc = HtmlOptions::new().full_document(true);
        let _with_prefix = HtmlOptions::new().with_class_prefix("custom");
        let _default = HtmlOptions::default();
    }

    #[test]
    fn html_options_chaining() {
        let options = HtmlOptions::new()
            .with_title("Test Survey")
            .with_styles(true)
            .full_document(true)
            .with_class_prefix("my-form");

        assert_eq!(options.title, Some("Test Survey".to_string()));
        assert!(options.include_styles);
        assert!(options.full_document);
        assert_eq!(options.class_prefix, "my-form");
    }
}
