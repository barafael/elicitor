# derive-wizard

Generate interactive CLI wizards or UIs by annotating Rust types. It's like magic!

## Quick Start

```rust,ignore
use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
struct ServerConfig {
    #[prompt("Server host:")]
    host: String,

    #[prompt("Server port:")]
    port: u16,

    #[prompt("Enable SSL:")]
    use_ssl: bool,
}

fn main() {
    let config = ServerConfig::wizard_builder().build().unwrap();
    println!("{config:#?}");
}
```

That's it. The program will prompt you for each field interactively.

## Features

- **Multiple backends** — requestty (default), dialoguer, ratatui, egui, typst
- **Password input** — `#[mask]` hides user input
- **Multiline text** — `#[multiline]` opens the user's editor or displays a text area, depending on the backend
- **Validation** — `#[validate("fn_name")]` with custom validator functions
- **Numeric bounds** — `#[min(n)]` and `#[max(n)]` for integers and floats
- **Enum selection** — Variants become selectable options
- **Multi-select** — `Vec<Enum>` allows multiple selections
- **Nested types** — Automatically flattened into the wizard interview flow
- **Suggestions** — Pre-fill fields with defaults users can edit
- **Assumptions** — Skip questions entirely with fixed values

## Backends

derive-wizard supports multiple (T)UI backends.

Enable backends via Cargo features:

```toml
[dependencies]
derive-wizard = { version = "0.5", features = ["dialoguer-backend"] }
```


### requestty (default)

Cross-platform terminal prompts with rich formatting.

```rust,ignore
let config = ServerConfig::wizard_builder().build()?;
```

### dialoguer

Simple, lightweight terminal dialogs.

```rust,ignore
use derive_wizard::DialoguerBackend;

let config = ServerConfig::wizard_builder()
    .with_backend(DialoguerBackend::new())
    .build()?;
```

### ratatui

Full TUI with keyboard navigation, progress bar, and styling.

```rust,ignore
use derive_wizard::RatatuiBackend;

let config = ServerConfig::wizard_builder()
    .with_backend(RatatuiBackend::new())
    .build()?;
```

### egui

Desktop GUI using immediate-mode rendering.

```rust,ignore
use derive_wizard::EguiBackend;

let config = ServerConfig::wizard_builder()
    .with_backend(EguiBackend::new("Window Title"))
    .build()?;
```

### typst

Generate PDF forms from your struct definitions. Not technically a backend, but hey. It's pretty fun.

```rust,ignore
let form_markup = ServerConfig::to_typst_form(Some("Configuration Form"));
std::fs::write("form.typ", form_markup)?;
```

## Proc-Macro Attributes

| Attribute            | Applies To  | Description                                                    |
|----------------------|-------------|----------------------------------------------------------------|
| `#[prompt("...")]`   | Fields      | The message shown to the user (required for non-builtin types) |
| `#[mask]`            | `String`    | Hide input for passwords                                       |
| `#[multiline]`       | `String`    | Open text editor for long input                                |
| `#[validate("fn")]`  | Any field   | Custom validation function                                     |
| `#[min(n)]`          | Numeric     | Minimum allowed value                                          |
| `#[max(n)]`          | Numeric     | Maximum allowed value                                          |
| `#[prelude("...")]`  | Struct/Enum | Message shown before the wizard starts                         |
| `#[epilogue("...")]` | Struct/Enum | Message shown after completion                                 |

## Supported Types

| Type                                       | Question Style                 |
|--------------------------------------------|--------------------------------|
| `String`                                   | Text input                     |
| `bool`                                     | Yes/No confirmation            |
| `i8`–`i128`, `u8`–`u128`, `isize`, `usize` | Integer input                  |
| `f32`, `f64`                               | Float input                    |
| `PathBuf`                                  | Text input (path)              |
| Enum                                       | Single selection from variants |
| `Vec<Enum>`                                | Multi-select from variants     |
| Nested struct                              | Questions inlined into flow    |

## Examples

### Password Input

```rust,ignore
#[derive(Debug, Wizard)]
struct LoginForm {
    #[prompt("Username:")]
    username: String,

    #[prompt("Password:")]
    #[mask]
    password: String,
}
```

### Multiline Text

```rust,ignore
#[derive(Debug, Wizard)]
struct Article {
    #[prompt("Title:")]
    title: String,

    #[prompt("Content:")]
    #[multiline]
    body: String,
}
```

### Validation

```rust,ignore
use derive_wizard::{Wizard, Answers};

#[derive(Debug, Wizard)]
struct Account {
    #[prompt("Email address:")]
    #[validate("validate_email")]
    email: String,
}

fn validate_email(input: &str, _answers: &Answers) -> Result<(), String> {
    if input.contains('@') && input.contains('.') {
        Ok(())
    } else {
        Err("Invalid email format".into())
    }
}
```

### Enum Selection

```rust,ignore
#[derive(Debug, Wizard)]
enum Transport {
    Car,
    Bike(#[prompt("Bike manufacturer:")] String),
    Walk,
}

#[derive(Debug, Wizard)]
struct Trip {
    #[prompt("Destination:")]
    destination: String,

    #[prompt("How will you travel?")]
    transport: Transport,
}
```

### Nested Structs

```rust,ignore
#[derive(Debug, Wizard)]
struct Address {
    #[prompt("Street:")]
    street: String,

    #[prompt("City:")]
    city: String,
}

#[derive(Debug, Wizard)]
struct UserProfile {
    #[prompt("Name:")]
    name: String,

    #[prompt("Home address:")]
    address: Address,
}
```

## Builder API

### Suggestions (Editable Defaults)

Pre-fill fields with values users can modify:

```rust,ignore
// From an existing instance
let updated = ServerConfig::wizard_builder()
    .with_suggestions(existing_config)
    .build()?;

// For specific fields
let config = ServerConfig::wizard_builder()
    .suggest_field("host", "localhost".to_string())
    .suggest_field("port", 8080)
    .build()?;
```

### Assumptions (Skip Questions)

Fix values without asking:

```rust,ignore
let config = ServerConfig::wizard_builder()
    .assume_field("use_ssl", true)
    .assume_field("port", 443)
    .build()?;  // Only asks about "host"
```

### References to Nested Fields

Use the `field!` macro for type-safe nested field paths:

```rust,ignore
use derive_wizard::field;

let profile = UserProfile::wizard_builder()
    .suggest_field(field!(name), "John".to_string())
    .assume_field(field!(UserProfile::address::city), "Boston".to_string())
    .build()?;
```

## License

MIT OR Apache-2.0
