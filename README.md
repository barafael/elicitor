# elicitor

Derive interactive surveys from Rust types.

This workspace contains the elicitor crates:

| Crate                             | Description                               |
|-----------------------------------|-------------------------------------------|
| [elicitor](elicitor/)             | Main crate with `#[derive(Survey)]` macro |
| [elicitor-types](elicitor-types/) | Core data structures and traits           |
| [elicitor-macro](elicitor-macro/) | Procedural macro implementation           |

**Backends:**

| Crate                                                   | Description               |
|---------------------------------------------------------|---------------------------|
| [elicitor-wizard-dialoguer](elicitor-wizard-dialoguer/) | CLI prompts via dialoguer |
| [elicitor-wizard-requestty](elicitor-wizard-requestty/) | CLI prompts via requestty |
| [elicitor-wizard-ratatui](elicitor-wizard-ratatui/)     | Terminal UI wizard        |
| [elicitor-form-ratatui](elicitor-form-ratatui/)         | Terminal UI form          |
| [elicitor-form-egui](elicitor-form-egui/)               | Native GUI form           |

**Document generators:**

| Crate                                     | Description           |
|-------------------------------------------|-----------------------|
| [elicitor-doc-html](elicitor-doc-html/)   | HTML form output      |
| [elicitor-doc-latex](elicitor-doc-latex/) | LaTeX document output |

See the [elicitor README](elicitor/README.md) for usage documentation.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
