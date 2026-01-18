# elicitor

Derive interactive surveys from Rust types.

Elicitor generates survey definitions from structs and enums using a procedural macro.
The resulting surveys can be presented through different backends:
terminal wizards, terminal forms, or graphical interfaces.

Documents to fill out can also be generated from a survey definition.

## Basic Usage

```rust
use elicitor::Survey;

#[derive(Survey, Debug)]
struct UserProfile {
    #[ask("What is your name?")]
    #[validate(name_rules)]
    name: String,

    #[ask("How old are you?")]
    #[min(0)]
    #[max(150)]
    age: u32,

    #[ask("Receive notifications?")]
    notifications: bool,
}

fn name_rules(name: &str) -> Result<(), String> {
    if name.len() < 3 {
        Err("Name too short".to_string())
    } else {
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let profile: UserProfile = UserProfile::builder()
        .run(elicitor_wizard_dialoguer::DialoguerWizard::new())?;

    println!("Created profile: {:?}", profile);
    Ok(())
}
```

Add to your `Cargo.toml`:

```toml
[dependencies]
elicitor = "0.6"
elicitor-wizard-dialoguer = "0.6"  # or another backend
```

## Attributes

### On types

| Attribute                     | Purpose                                     |
|-------------------------------|---------------------------------------------|
| `#[prelude("...")]`           | Message shown before the survey             |
| `#[epilogue("...")]`          | Message shown after completion              |
| `#[validate(fn_name)]`        | Composite validator for cross-field checks  |

### On fields

| Attribute                 | Purpose                             |
|---------------------------|-------------------------------------|
| `#[ask("...")]`           | Prompt text shown to the user       |
| `#[mask]`                 | Hide input (passwords)              |
| `#[multiline]`            | Multi-line text input               |
| `#[validate(fn_name)]`    | Field-level validation              |
| `#[min(n)]` / `#[max(n)]` | Numeric bounds                      |
| `#[multiselect]`          | Multi-select for `Vec<Enum>` fields |

## Supported Types

- **Primitives**: `String`, `bool`, integers (`i8`..`i64`, `u8`..`u64`), floats (`f32`, `f64`)
- **Collections**: `Vec<T>` where T is a primitive or enum
- **Optional**: `Option<T>` for any supported T
- **Nested structs**: Types that also derive `Survey`
- **Enums**: Unit variants, tuple variants, and struct variants
- **Path types**: `PathBuf`

## Enums

Enums become selection questions. The user picks a variant, then fills in any associated data.

```rust
#[derive(Survey, Debug)]
enum ContactMethod {
    Email {
        #[ask("Email address:")]
        address: String,
    },
    Phone(#[ask("Phone number:")] String),
    None,
}

#[derive(Survey, Debug)]
struct Contact {
    #[ask("Name:")]
    name: String,

    #[ask("Preferred contact method:")]
    method: ContactMethod,
}
```

For multi-select (choosing multiple variants), use `Vec<Enum>` with `#[multiselect]`:

```rust
#[derive(Survey, Debug)]
enum Feature {
    DarkMode,
    Notifications,
    Analytics (#[ask("Analytics ID:")] u32),
}

#[derive(Survey, Debug)]
struct Preferences {
    #[ask("Enable features:")]
    #[multiselect]
    features: Vec<Feature>,
}
```

As you can see, enums can have associated data, which is collected separately from the multiselect itself.

## Validation

Field-level validators receive the current value and all collected responses:

```rust
fn validate_email(
    value: &elicitor::ResponseValue,
    _responses: &elicitor::Responses,
    _path: &elicitor::ResponsePath,
) -> Result<(), String> {
    let s = value.as_string().unwrap_or("");
    if s.contains('@') {
        Ok(())
    } else {
        Err("Invalid email address".into())
    }
}

#[derive(Survey)]
struct Account {
    #[ask("Email:")]
    #[validate("validate_email")]
    email: String,
}
```

Composite validators check relationships between fields:

```rust
fn passwords_match(responses: &elicitor::Responses) -> HashMap<ResponsePath, String> {
    let mut errors = HashMap::new();
    let pw = responses.get_string(&ResponsePath::new("password"));
    let confirm = responses.get_string(&ResponsePath::new("confirm"));
    
    if let (Ok(pw), Ok(confirm)) = (pw, confirm) {
        if pw != confirm {
            errors.insert(ResponsePath::new("confirm"), "Passwords must match".into());
        }
    }
    errors
}

#[derive(Survey)]
#[validate("passwords_match")]
struct PasswordForm {
    #[ask("Password:")]
    #[mask]
    password: String,

    #[ask("Confirm:")]
    #[mask]
    confirm: String,
}
```

## Builder Pattern for assumptions and suggestions

You can pre-fill values as suggestions or skip questions which have assumed answers.
Details depend on the backend.

**Suggestions** pre-fill fields with editable defaults:

```rust
let profile = UserProfile::builder()
    .suggest_name("Alice")
    .suggest_age(30)
    .run(backend)?;
```

**Assumptions** skip questions entirely:

```rust
let profile = UserProfile::builder()
    .assume_name("System User")  // User won't be prompted
    .run(backend)?;
```

**Bulk suggestions** from an existing instance:

```rust
let existing = load_profile()?;
let updated = UserProfile::builder()
    .with_suggestions(&existing)
    .run(backend)?;
```

## Backends

Backends present the survey to users. Each is a separate crate.

| Crate                       | Style  | Description                         |
|-----------------------------|--------|-------------------------------------|
| `elicitor-wizard-dialoguer` | Wizard | CLI prompts via dialoguer           |
| `elicitor-wizard-requestty` | Wizard | CLI prompts via requestty           |
| `elicitor-wizard-ratatui`   | Wizard | Terminal UI, one question at a time |
| `elicitor-form-ratatui`     | Form   | Terminal UI, all fields visible     |
| `elicitor-form-egui`        | Form   | Native GUI via egui                 |

**Wizard-style** backends ask one question at a time. **Form-style** backends show all fields simultaneously.

### Document Generators

These crates generate static documents from survey definitions:

| Crate                | Output         |
|----------------------|----------------|
| `elicitor-doc-html`  | HTML form      |
| `elicitor-doc-latex` | LaTeX document |

## Testing

Use `TestBackend` for unit tests:

```rust
#[test]
fn test_profile_creation() {
    let profile: UserProfile = UserProfile::builder()
        .run(
            elicitor::TestBackend::new()
                .with_response("name", "Test User")
                .with_response("age", 25)
                .with_response("notifications", true)
        )
        .unwrap();

    assert_eq!(profile.name, "Test User");
    assert_eq!(profile.age, 25);
}
```

## Architecture

The crate is split into three parts:

- **elicitor-types**: Core data structures (`SurveyDefinition`, `Question`, `Responses`, traits)
- **elicitor-macro**: The `#[derive(Survey)]` procedural macro
- **elicitor**: Facade crate that re-exports both

Users only need to depend on `elicitor`. The macro generates code that uses types from `elicitor_types`, which are re-exported through the main crate.

See [docs/architecture.md](../docs/architecture.md) for details.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
