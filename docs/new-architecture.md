# Derive-Survey Architecture

This document describes the architecture of the **derive-survey** crate.

A "survey" is a structured collection of questions. It's presentation-agnostic.
It can be viewed as a
sequential interview or a
fill-in form or just
data for generating a document.

## Usage Example

For illustration, this is what we are going for:

```rust
use derive_survey::{Survey, SurveyBuilder};

#[derive(Survey, Debug)]
#[prelude("A journey begins...!")]
#[epilogue("Good luck.")]
struct MySurvey {
    #[ask("What is your name?")]
    #[validate("is_valid_name")]
    name: String,

    #[ask("What's the secret passphrase?")]
    #[mask]
    passphrase: String,

    #[ask("How old are you?")]
    #[min(18)]
    #[max(233)]
    age: u32,

    #[ask("What is your role?")]
    role: Role,

    #[ask("Pick your inventory:")]
    #[multiselect]
    #[validate("is_within_starting_budget")]
    inventory: Vec<Item>,
}

#[derive(Survey, Debug)]
enum Role {
    Streetfighter,
    Mage,
    Archer,
    Thief,
    Other(#[ask("What then?!")] String)
}

#[derive(Survey, Debug)]
enum Item {
    #[ask("Sword (value: 80)")]
    Sword,

    #[ask("Shield (value: 50)")]
    Shield,

    #[ask("Potion (value: 20)")]
    Potion,

    #[ask("Scroll (value: 10)")]
    Scroll,

    #[ask("Chewing Gum (value: 2 * quantity)")]
    ChewingGum {
        flavor: String,
        #[ask("Quando?")]
        quantity: u32,
    },
}

impl Item {
    fn value(&self) -> u32 {
        match self {
            Item::Sword => 80,
            Item::Shield => 50,
            Item::Potion => 20,
            Item::Scroll => 10,
            Item::ChewingGum { flavor: _, quantity } => 2 * quantity,
        }
    }
}

fn is_valid_name(name: &str) -> Result<(), String> {
    if name.len() > 2 && name.len() < 100 {
        Ok(())
    } else {
        Err("Name must be between 3 and 99 characters".to_string())
    }
}

fn is_within_starting_budget(inventory: &Vec<Item>) -> Result<(), String> {
    let total_value = inventory.iter().map(Item::value).sum::<u32>();

    if total_value <= 120 {
        Ok(())
    } else {
        Err("Inventory value exceeds starting budget".to_string())
    }
}

fn main() {
    // run the survey with the requestty backend
    let survey_result = MySurvey::builder()
        .run(RequesttyWizard::new())
        .unwrap();

    println!("{:#?}", survey_result);
}
```

### Who-is-who

| Element         | Name                |
|-----------------|---------------------|
| Crate           | `derive-survey`     |
| Derive macro    | `#[derive(Survey)]` |
| Trait           | `Survey`            |
| Data structure  | `SurveyDefinition`  |
| Individual item | `Question`          |
| Item variants   | `QuestionKind`      |
| Collected data  | `Responses`         |
| Builder         | `SurveyBuilder`     |
| Backend trait   | `SurveyBackend`     |

### Proc-macro Attributes

| Attribute                 | Purpose                           |
|---------------------------|-----------------------------------|
| `#[ask("...")]`           | The prompt text shown to the user |
| `#[mask]`                 | Hide input (for passwords)        |
| `#[multiline]`            | Open text editor / show textarea  |
| `#[validate("fn")]`       | Custom validation function        |
| `#[min(n)]` / `#[max(n)]` | Numeric bounds                    |
| `#[prelude("...")]`       | Message before survey starts      |
| `#[epilogue("...")]`      | Message after survey completes    |

## Two Interaction Models

The design supports two fundamentally different interaction paradigms:

### Sequential (Wizard-style)

**Backends:** requestty, dialoguer, ratatui-wizard

**Characteristics:**

- One question at a time
- User answers, then moves to next
- Linear flow, no back-navigation
- Validation per-field before proceeding
- Natural for CLI prompts

### Form-style

**Backends:** ratatui-form, egui-form

**Characteristics:**

- All fields visible simultaneously
- User can fill in any order
- Jump between fields freely
- Validation as user types, submit only possible once inputs are valid
- All validation errors are shown immediately and simultaneously
- Inter-field conditions (such as "passwords entered must match") are validated as user types
- Natural for GUIs and TUIs

## Crate Structure

### Core Crates

Users only need to depend on `derive-survey` for access to `#[derive(Survey)]`.
The macro crate cannot export types, so the split is necessary.
The main crate re-exports everything so generated code works without users adding `derive-survey-macro` and `derive-survey-types` manually.

The main crate does NOT include any backend implementations (except a private `TestBackend` for testing).

### Backend Crates (independent, user-chosen)

Pattern: `derive-{library}-{style}`

Wizard-style examples:

```
derive-requestty-wizard     # CLI prompts via requestty
derive-dialoguer-wizard     # CLI prompts via dialoguer
derive-ratatui-wizard       # TUI wizard with step-by-step flow
```

Form-style examples:

```
derive-ratatui-form         # TUI form with field navigation
derive-egui-form            # GUI form via egui
```

### Output Crates

Output crates are not backends — they don't collect responses.
They transform a `SurveyDefinition` into a document format.
They depend on `derive-survey` but don't implement `SurveyBackend`.

For example:

```
derive-typst-document       # Generates .typ markup
derive-html-document        # generates .html markup
```

```rust
// In derive-typst-document
use derive_survey::Survey;

pub fn to_typst<T: Survey>(title: Option<&str>) -> String {
    let definition = T::survey();
    generate_typst_markup(&definition, title)
}
```

### Backend Autonomy

Backends decide how to present a `SurveyDefinition`.

```rust
pub trait SurveyBackend {
    type Error: Into<anyhow::Error>;

    fn collect(
        &self,
        definition: &SurveyDefinition,
        validate: &dyn Fn(&ResponsePath, &Responses) -> Result<(), String>,
    ) -> Result<Responses, Self::Error>;
}
```

A wizard backend iterates through questions sequentially. A form backend renders all fields at once.
The trait doesn't care — it just takes a `SurveyDefinition` and returns `Responses`.

Each backend crate:

1. Depends on `derive-survey` for the `SurveyDefinition` structure and `SurveyBackend` trait
2. Decides independently how to present the survey to the user
3. Can implement `SurveyBackend` however it sees fit
4. Is responsible for its own dependencies (ratatui, egui, etc.)

```rust
// In derive-requestty-wizard
use derive_survey::{SurveyDefinition, Responses, ResponsePath, SurveyBackend};

pub struct RequesttyWizard;

impl SurveyBackend for RequesttyWizard {
    type Error = RequesttyError;

    fn collect(
        &self,
        definition: &SurveyDefinition,
        validate: &dyn Fn(&ResponsePath, &Responses) -> Result<(), String>,
    ) -> Result<Responses, Self::Error> {
        // Sequential prompting - this backend's choice
        for question in definition.questions() {
            // prompt one at a time, validate, retry on error...
        }
    }
}
```

Or Egui:

```rust
// In derive-egui-form
use derive_survey::{SurveyDefinition, Responses, ResponsePath, SurveyBackend};

pub struct EguiForm { /* ... */ }

impl SurveyBackend for EguiForm {
    type Error = EguiFormError;

    fn collect(
        &self,
        definition: &SurveyDefinition,
        validate: &dyn Fn(&ResponsePath, &Responses) -> Result<(), String>,
    ) -> Result<Responses, Self::Error> {
        // Render all fields at once - this backend's choice
        render_form(definition.questions());
        // validate on submit, show errors, retry...
    }
}
```

## Dependency Graph

### User Application

The user depends on derive-survey for the proc-macro, and on a selected backend crate.

```
user-app/
├── Cargo.toml
│   └── [dependencies]
│       ├── derive-survey = "1"
│       ├── derive-requestty-wizard = "1"  (example backend choice)
│       └── derive-typst-document = "1"    (optional output crate)
│
```

Meanwhile, the dependency tree of `derive-survey` itself is as follows:

```
derive-survey
├── Cargo.toml
│   └── [dependencies]
│       ├── derive-survey-types = "1.0"
│       └── derive-survey-macro = "1.0"
│
├── lib.rs
│   ├── pub use derive_survey_types::*;      // Re-export all types
│   ├── pub use derive_survey_macro::Survey; // Re-export #[derive(Survey)]
│   ├── mod builder;                         // SurveyBuilder (allows setting suggestions and assumptions)
│   └── mod test_backend;                    // TestBackend (private, for testing)
│
└── (dependencies)
    │
    ├─ derive-survey-types/
    │  ├── Cargo.toml
    │  │   └── [dependencies]
    │  │       └── (does not depend on derive-survey)
    │  │
    │  └── lib.rs
    │      ├── struct SurveyDefinition { ... }
    │      ├── struct Question { ... }
    │      ├── enum QuestionKind { ... }
    │      ├── struct Responses { ... }
    │      ├── struct ResponsePath { ... }
    │      ├── enum ResponseValue { ... }
    │      ├── enum SurveyError { ... }
    │      ├── trait Survey { ... }
    │      └── trait SurveyBackend { ... }
    │
    └─ derive-survey-macro/
       ├── Cargo.toml
       │   └── [dependencies]
       │       ├── derive-survey-types = "1.0"
       │       ├── syn = "2.0"
       │       ├── quote = "1.0"
       │       └── proc-macro2 = "1.0"
       │
       └── lib.rs
           └── #[proc_macro_derive(Survey, attributes(...))]
               // Generates code using types from derive-survey-types
               // (re-exported via derive-survey)
```

And the ecosystem:

```
derive-requestty-wizard/
├── Cargo.toml
│   └── [dependencies]
│       ├── derive-survey = "1.0"
│       └── requestty = "..."
│
└── lib.rs
    ├── use derive_survey::{SurveyBackend, SurveyDefinition, Responses, ...};
    ├── pub struct RequesttyWizard;
    └── impl SurveyBackend for RequesttyWizard { ... }

derive-typst-document/
├── Cargo.toml
│   └── [dependencies]
│       └── derive-survey = "1.0"
│
└── lib.rs
    ├── use derive_survey::{Survey, SurveyDefinition};
    └── pub fn to_typst<T: Survey>(...) -> String { ... }
```

Other Backend and Output Crates (following the same pattern):

**Wizard-style backends:**

- `derive-dialoguer-wizard` → depends on derive-survey, dialoguer
- `derive-ratatui-wizard` → depends on derive-survey, ratatui

**Form-style backends:**

- `derive-ratatui-form` → depends on derive-survey, ratatui
- `derive-egui-form` → depends on derive-survey, egui

**Output crates:**

- `derive-html-document` → depends on derive-survey

Key Dependency Rules:
─────────────────────

1. derive-survey-types
   - No dependencies on other derive-survey crates
   - Pure data structures and traits
   - Foundation layer

2. derive-survey-macro
   - Depends on: derive-survey-types
   - Proc-macro that generates Survey implementations
   - Generated code uses types that will be re-exported by derive-survey

3. derive-survey (facade)
   - Depends on: derive-survey-types, derive-survey-macro
   - Re-exports everything from both
   - Adds SurveyBuilder and TestBackend
   - Single dependency for users

4. Backend crates
   - Depend on: derive-survey (gets types via re-exports)
   - Each implements SurveyBackend trait
   - No interdependencies between backends
   - Users choose which to include

5. Output crates
   - Depend on: derive-survey
   - Transform SurveyDefinition → documents
   - Do NOT implement SurveyBackend (not interactive)

6. User applications
   - Always depend on: derive-survey
   - Choose backend(s): derive-{lib}-{wizard|form}
   - Choose output format: derive-{format}-document

## Core Types

### SurveyDefinition

The top-level structure containing all questions and metadata:

```rust
pub struct SurveyDefinition {
    /// Optional message shown before the survey starts
    pub prelude: Option<String>,

    /// All questions in the survey (may contain nested AllOf/OneOf/AnyOf)
    pub questions: Vec<Question>,

    /// Optional message shown after the survey completes
    pub epilogue: Option<String>,
}
```

### ResponsePath

Typed paths instead of dot-separated strings:

```rust
/// A path to a response value, e.g., ["address", "street"]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResponsePath {
    segments: Vec<String>,
}

impl ResponsePath {
    pub fn root(name: impl Into<String>) -> Self {
        Self { segments: vec![name.into()] }
    }

    pub fn child(mut self, name: impl Into<String>) -> Self {
        self.segments.push(name.into());
        self
    }

    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    /// Returns a new path with the given prefix removed, if it matches
    pub fn strip_prefix(&self, prefix: &str) -> Option<Self> {
        if self.segments.first().map(|s| s.as_str()) == Some(prefix) {
            Some(Self { segments: self.segments[1..].to_vec() })
        } else {
            None
        }
    }

    /// Converts the path to a dot-separated string for display/debugging
    pub fn as_str(&self) -> String {
        self.segments.join(".")
    }
}
```

### Responses

Uses `ResponsePath` as keys:

```rust
pub struct Responses {
    values: HashMap<ResponsePath, ResponseValue>,
}

pub enum ResponseValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    ChosenVariant(usize),        // For OneOf questions (single enum variant selection)
    ChosenVariants(Vec<usize>),  // For AnyOf questions (multi-select enum variants)
}

impl Responses {
    pub fn get(&self, path: &ResponsePath) -> Option<&ResponseValue>;
    pub fn insert(&mut self, path: ResponsePath, value: ResponseValue);

    /// Filters responses to only those with the given path prefix, removing the prefix from keys
    pub fn filter_prefix(&self, prefix: &ResponsePath) -> Self {
        let mut filtered = Responses { values: HashMap::new() };
        for (path, value) in &self.values {
            if path.segments().starts_with(prefix.segments()) {
                let new_path = ResponsePath {
                    segments: path.segments()[prefix.segments().len()..].to_vec()
                };
                filtered.insert(new_path, value.clone());
            }
        }
        filtered
    }

    // Convenience methods
    pub fn get_string(&self, path: &ResponsePath) -> Result<&str, Error>;
    pub fn get_int(&self, path: &ResponsePath) -> Result<i64, Error>;
    pub fn get_float(&self, path: &ResponsePath) -> Result<f64, Error>;
    pub fn get_bool(&self, path: &ResponsePath) -> Result<bool, Error>;
    pub fn get_chosen_variant(&self, path: &ResponsePath) -> Result<usize, Error>;
    pub fn get_chosen_variants(&self, path: &ResponsePath) -> Result<&[usize], Error>;
    // etc.
}
```

### Question

Individual questions in the survey, with mutation methods for the builder:

```rust
pub struct Question {
    path: ResponsePath,
    ask: String,
    kind: QuestionKind,
    default: DefaultValue,
}

impl Question {
    pub fn path(&self) -> &ResponsePath;
    pub fn ask(&self) -> &str;
    pub fn kind(&self) -> &QuestionKind;
    pub fn default(&self) -> &DefaultValue;

    /// Set a suggested default value (user can modify)
    pub fn set_suggestion(&mut self, value: impl Into<ResponseValue>) {
        self.default = DefaultValue::Suggested(value.into());
    }

    /// Set an assumed value (question is skipped entirely)
    pub fn set_assumption(&mut self, value: impl Into<ResponseValue>) {
        self.default = DefaultValue::Assumed(value.into());
    }
}

pub enum DefaultValue {
    None,
    Suggested(ResponseValue),  // Pre-filled, user can modify
    Assumed(ResponseValue),    // Question skipped, value used directly
}
```

These mutation methods are used by `SurveyBuilder` to apply partial suggestions and assumptions before passing the survey to a backend.

### QuestionKind

The `QuestionKind` enum represents the different types of questions:

```rust
pub enum QuestionKind {
    /// No data to collect (unit enum variants)
    None,

    /// Single-line text input
    Input(InputQuestion),

    /// Multi-line text input (opens editor or textarea)
    Multiline(MultilineQuestion),

    /// Masked input for passwords
    Masked(MaskedQuestion),

    /// Integer input with optional min/max bounds
    Int(IntQuestion),

    /// Floating-point input with optional min/max bounds
    Float(FloatQuestion),

    /// Yes/no confirmation
    Confirm(ConfirmQuestion),

    /// Select any number of options from a list (Vec<Enum>)
    AnyOf(AnyOfQuestion),

    /// A group of questions — answer all (nested structs, struct variants)
    AllOf(Vec<Question>),

    /// Choose one variant — pick one, then answer its questions (enums)
    OneOf(Vec<Variant>),
}

/// A variant in a OneOf (enum)
pub struct Variant {
    /// Variant name for display ("Male", "Female", "Other")
    pub name: String,
    /// What to collect for this variant (None for unit variants, Input for newtype, AllOf for struct variants, OneOf for nested enums)
    pub kind: QuestionKind,
}
```

The variant's index is its position in the `Vec<Variant>`.

**AllOf**, **OneOf**, and **AnyOf** are structural variants that enable nested surveys:

- **`AllOf(Vec<Question>)`**: Groups questions together — all are answered. Used for nested structs and struct enum variants.

- **`OneOf(Vec<Variant>)`**: Choose one variant from a list. User picks exactly one, then answers that variant's questions. Used for enums. The selected variant's index is stored as `selected_variant`.

- **`AnyOf(AnyOfQuestion)`**: Choose any number of variants from a list. User picks zero or more, then answers follow-up questions for each selected variant. Used for `Vec<Enum>`. Selected indices are stored as `selected_variants` (ChosenVariants).

Example for an enum with a newtype variant:

```rust
#[derive(Survey)]
enum Gender {
    Male,
    Female,
    Other(#[ask("Please specify:")] String),
}

// Generates:
OneOf(vec![
    Variant { name: "Male", kind: None },
    Variant { name: "Female", kind: None },
    Variant { name: "Other", kind: Input(InputQuestion { ... }) },
])
// The #[ask] on the newtype field provides the prompt shown after selecting "Other"
```

Response paths for newtype variants use index `"0"`:

```
"gender.selected_variant" -> ChosenVariant(2)
"gender.0"                -> "Non-binary"
```

Example for struct variants (named fields use field names in paths):

```rust
#[derive(Survey)]
enum Contact {
    Email {
        #[ask("Email address:")]
        address: String,
        #[ask("Verified?")]
        verified: bool,
    },
    Phone(#[ask("Phone number:")] String),
}

// Generates:
OneOf(vec![
    Variant {
        name: "Email",
        kind: AllOf(vec![
            Question { path: ["address"], ask: "Email address:", kind: Input(...) },
            Question { path: ["verified"], ask: "Verified?", kind: Confirm(...) },
        ])
    },
    Variant {
        name: "Phone",
        kind: Input(...)  // prompt comes from #[ask] on the field
    },
])
```

Response paths for struct variants use field names:

```
"contact.selected_variant" -> ChosenVariant(0)
"contact.address"          -> "alice@example.com"
"contact.verified"         -> true
```

Nesting is fully supported — a `Variant`'s `kind` can be `OneOf` containing more `Variant`s, enabling arbitrarily deep enum nesting.

### AnyOf Questions

AnyOf allows users to select any number of options from a list, optionally with follow-up questions for each selected option. It's used with `Vec<T>` fields where `T` is an enum:

```rust
pub struct AnyOfQuestion {
    /// The available variants (reuses the same Variant struct as OneOf)
    pub variants: Vec<Variant>,
    /// Default selected indices (if any)
    pub defaults: Vec<usize>,
}
```

AnyOf reuses the `Variant` struct, enabling variants with data just like OneOf. The difference: OneOf picks exactly one, AnyOf picks any number.

#### Simple Example (Unit Variants)

```rust
#[derive(Survey)]
enum Topping {
    Cheese,
    Pepperoni,
    Mushrooms,
    Olives,
}

#[derive(Survey)]
struct PizzaOrder {
    #[ask("Select toppings:")]
    toppings: Vec<Topping>,
}

// Generates:
AnyOf(AnyOfQuestion {
    variants: vec![
        Variant { name: "Cheese", kind: None },
        Variant { name: "Pepperoni", kind: None },
        Variant { name: "Mushrooms", kind: None },
        Variant { name: "Olives", kind: None },
    ],
    defaults: vec![],
})
```

Response storage for unit variants — selected indices stored as `ChosenVariants`:

```rust
// User selects Cheese and Mushrooms
"toppings.selected_variants" -> ChosenVariants(vec![0, 2])
```

Reconstruction:

```rust
fn from_responses(responses: &Responses) -> Self {
    let indices = responses.get_chosen_variants(&ResponsePath::from("toppings.selected_variants")).unwrap();
    let toppings = indices.iter()
        .map(|&i| match i {
            0 => Topping::Cheese,
            1 => Topping::Pepperoni,
            2 => Topping::Mushrooms,
            3 => Topping::Olives,
            _ => unreachable!(),
        })
        .collect();
    PizzaOrder { toppings }
}
```

#### Variants with Data

AnyOf supports variants with data. Each selected variant can have follow-up questions:

```rust
#[derive(Survey)]
enum Feature {
    DarkMode,
    Notifications {
        #[ask("Email notifications?")]
        email: bool,
        #[ask("Push notifications?")]
        push: bool,
    },
    CustomTheme(#[ask("Theme color:")] String),
}

#[derive(Survey)]
struct Preferences {
    #[ask("Select features:")]
    features: Vec<Feature>,
}

// Generates:
AnyOf(AnyOfQuestion {
    variants: vec![
        Variant { name: "DarkMode", kind: None },
        Variant { name: "Notifications", kind: AllOf(vec![
            Question { path: ["email"], ask: "Email notifications?", kind: Confirm(...) },
            Question { path: ["push"], ask: "Push notifications?", kind: Confirm(...) },
        ])},
        Variant { name: "CustomTheme", kind: Input(...) },
    ],
    defaults: vec![],
})
```

**Backend flow:** User sees checkboxes for all variants. After selection, backend asks follow-up questions for each selected variant that has `kind != None`.

Response storage uses the variant index as a path segment to namespace each selected variant's data:

```rust
// User selects DarkMode and Notifications (with email=true, push=false)
"features.selected_variants" -> ChosenVariants(vec![0, 1])
"features.1.email"           -> Bool(true)
"features.1.push"            -> Bool(false)
```

If user also selects CustomTheme:

```rust
"features.selected_variants" -> ChosenVariants(vec![0, 1, 2])
"features.1.email"           -> Bool(true)
"features.1.push"            -> Bool(false)
"features.2.0"               -> String("blue")  // newtype uses "0" as field path
```

Reconstruction iterates selected indices and builds each variant:

```rust
fn from_responses(responses: &Responses) -> Self {
    let indices = responses.get_chosen_variants(&ResponsePath::from("features.selected_variants")).unwrap();
    let features = indices.iter()
        .map(|&i| {
            let base = ResponsePath::root("features").child(i.to_string());
            match i {
                0 => Feature::DarkMode,
                1 => Feature::Notifications {
                    email: responses.get_bool(&base.clone().child("email")).unwrap(),
                    push: responses.get_bool(&base.child("push")).unwrap(),
                },
                2 => Feature::CustomTheme(
                    responses.get_string(&base.child("0")).unwrap().to_string()
                ),
                _ => unreachable!(),
            }
        })
        .collect();
    Preferences { features }
}
```

**Note:** Backends present this as a two-phase interaction: first select variants (checkboxes), then answer follow-up questions for selected variants with data. Form backends may show all variant questions but only validate/submit the selected ones.

### Survey Trait

```rust
pub trait Survey: Sized {
    /// Returns the survey structure (questions, prompts, validation metadata)
    fn survey() -> SurveyDefinition;

    /// Reconstructs an instance from collected responses.
    /// This is infallible — the macro generates both survey() and from_responses(),
    /// guaranteeing they are consistent. If all questions are answered, reconstruction succeeds.
    fn from_responses(responses: &Responses) -> Self;

    /// Validates a field's value.
    /// Called by backends during input collection.
    /// The validator receives the path and queries the responses for the value.
    fn validate_field(path: &ResponsePath, responses: &Responses) -> Result<(), String>;

    /// Validates the entire survey (composite validators, inter-field conditions)
    /// Returns a map of path -> error message for all validation failures
    fn validate_all(responses: &Responses) -> HashMap<ResponsePath, String>;

    /// Returns a builder for running the survey
    fn builder() -> SurveyBuilder<Self> {
        SurveyBuilder::new()
    }
}
```

### SurveyBackend Trait

```rust
pub trait SurveyBackend {
    type Error: Into<anyhow::Error>;

    /// Collect responses for a survey.
    ///
    /// Returns Ok(responses) on success, Err on cancellation or backend failure.
    /// Validation is handled internally — this only returns when all fields are valid.
    fn collect(
        &self,
        definition: &SurveyDefinition,
        validate: &dyn Fn(&ResponsePath, &Responses) -> Result<(), String>,
    ) -> Result<Responses, Self::Error>;
}
```

The backend receives:

1. `SurveyDefinition` — the questions to ask
1. A validator function — to validate each response. The validator queries `Responses` by path,
   which allows validating composite fields (like `Vec<T>`) that don't have a single string value.

The `Into<anyhow::Error>` bound is permissive — it accepts custom error types, `anyhow::Error`, `std::io::Error`, and any `thiserror` derived error.

This keeps the backend generic. It doesn't need to know about `T`, just how to collect responses and validate them.

### Validation

There are two levels of validation:

**Field-level validation** (`#[validate("fn")]` on fields):

```rust
#[derive(Survey)]
struct Registration {
    #[ask("Email:")]
    #[validate("validate_email")]
    email: String,

    #[ask("Age:")]
    #[min(18)]
    age: u32,
}

fn validate_email(path: &ResponsePath, responses: &Responses) -> Result<(), String> {
    let value = responses.get_string(path).unwrap_or("");
    if value.contains('@') { Ok(()) } else { Err("Invalid email".into()) }
}
```

**Composite validation** (`#[validate("fn")]` on structs/enums):

```rust
#[derive(Survey)]
#[validate("validate_passwords_match")]
struct AccountSetup {
    #[ask("Password:")]
    #[mask]
    password: String,

    #[ask("Confirm password:")]
    #[mask]
    password_confirm: String,
}

/// Validates the partially-filled survey, returns errors keyed by path
fn validate_passwords_match(responses: &Responses) -> HashMap<ResponsePath, String> {
    let mut errors = HashMap::new();

    let pw = responses.get_string(&ResponsePath::root("password"));
    let confirm = responses.get_string(&ResponsePath::root("password_confirm"));

    if let (Ok(pw), Ok(confirm)) = (pw, confirm) {
        if pw != confirm {
            errors.insert(
                ResponsePath::root("password_confirm"),
                "Passwords do not match".into()
            );
        }
    }

    errors
}
```

### Validator Function Resolution

Validator functions are referenced by name in the `#[validate("fn")]` attribute. The macro generates a compile-time assertion to verify the function exists and has the correct signature:

```rust
// Generated by the macro for #[validate("validate_email")]
const _: fn(&ResponsePath, &str, &Responses) -> Result<(), String> = validate_email;
```

This catches typos and signature mismatches at compile time rather than runtime.

### Validation Flow

**Wizard backends (interactive, per-field):**

```rust
for question in survey.questions() {
    loop {
        let value = prompt_user_interactive(&question, |partial| {
            // Called on each keypress for live feedback
            T::validate_field(&question.path, partial, &responses)
        })?;

        // Validate on submit
        match T::validate_field(&question.path, &value, &responses) {
            Ok(()) => {
                responses.insert(question.path.clone(), value.into());
                break;  // Move to next question
            }
            Err(msg) => {
                show_error(&msg);
                // Loop: ask again
            }
        }
    }
}
```

**Form backends (live validation as user types):**

```rust
loop {
    render_form(&survey, &responses, &errors);

    match handle_input() {
        Keystroke(path, new_value) => {
            responses.insert(path.clone(), new_value.into());

            // Validate this field live
            if let Err(msg) = T::validate_field(&path, &new_value, &responses) {
                errors.insert(path, msg);
            } else {
                errors.remove(&path);
            }

            // Also run composite validators for inter-field conditions
            let composite_errors = T::validate_all(&responses);
            errors.extend(composite_errors);
        }

        Submit => {
            // Final validation of everything
            errors.clear();

            for question in survey.questions() {
                if let Some(value) = responses.get(&question.path) {
                    if let Err(msg) = T::validate_field(&question.path, value.as_str(), &responses) {
                        errors.insert(question.path.clone(), msg);
                    }
                }
            }

            errors.extend(T::validate_all(&responses));

            if errors.is_empty() {
                break;  // Success
            }
            // Stay in form, show all errors
        }
    }
}
```

### TestBackend

`derive-survey` includes a `TestBackend` for testing survey-enabled types without user interaction:

```rust
use derive_survey::{Survey, TestBackend};

#[derive(Debug, Survey)]
struct Config {
    #[ask("Host:")]
    host: String,
    #[ask("Port:")]
    port: u16,
}

#[test]
fn test_config_survey() {
    let config: Config = Config::builder()
        .run(
            TestBackend::new()
                .with_response("host", "localhost")
                .with_response("port", 8080)
        )
        .unwrap();

    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 8080);
}
```

## Suggestions and Assumptions

The survey system supports two ways to pre-fill values:

### Suggestions

A **suggestion** is a pre-filled default that the user can accept or modify. The field is still shown to the user, but with the suggested value already entered.

```rust
#[derive(Survey)]
struct ServerConfig {
    #[ask("Host:")]
    host: String,
    #[ask("Port:")]
    port: u16,
}

// Create a survey with suggestions from an existing instance
let existing = ServerConfig { host: "localhost".into(), port: 8080 };

let config: ServerConfig = ServerConfig::builder()
    .with_suggestions(&existing)  // Pre-fill all fields from existing instance
    .run(backend)
    .unwrap();

// Backend will show:
//   Host: [localhost]     <- user can edit or accept
//   Port: [8080]          <- user can edit or accept
```

Suggestions are stored in each `Question`'s default field. The builder's `with_suggestions(&self)` method walks the struct and populates question defaults from field values.

You can also set individual suggestions using **generated field-specific methods**:

```rust
let config: ServerConfig = ServerConfig::builder()
    .suggest_host("localhost")
    .suggest_port(8080)
    .run(backend)
    .unwrap();

// Backend will show:
//   Host: [localhost]     <- user can edit or accept
//   Port: [8080]          <- user can edit or accept
```

### Assumptions

An **assumption** is a value that skips the question entirely. The user is not prompted; the assumed value is used directly.

```rust
let config: ServerConfig = ServerConfig::builder()
    .assume_host("localhost")  // Skip host prompt, use "localhost"
    .run(backend)
    .unwrap();

// Backend will only show:
//   Port: [____]
```

### Generated Builder Methods

The `#[derive(Survey)]` macro generates a type-specific builder with field methods. For a struct like:

```rust
#[derive(Survey)]
struct UserProfile {
    #[ask("Name:")]
    name: String,

    #[ask("Age:")]
    age: u32,

    #[ask("Address:")]
    address: Address,
}

#[derive(Survey)]
struct Address {
    #[ask("Street:")]
    street: String,

    #[ask("City:")]
    city: String,
}
```

The macro generates:

```rust
// Generated by #[derive(Survey)]
pub struct UserProfileBuilder {
    suggestions: std::collections::HashMap<&'static str, ResponseValue>,
    assumptions: std::collections::HashMap<&'static str, ResponseValue>,
}

impl UserProfileBuilder {
    pub fn new() -> Self { /* ... */ }

    // === Suggestion methods (type-safe, IDE autocomplete) ===

    pub fn suggest_name(mut self, value: impl Into<String>) -> Self {
        self.suggestions.insert("name", ResponseValue::String(value.into()));
        self
    }

    pub fn suggest_age(mut self, value: u32) -> Self {
        self.suggestions.insert("age", ResponseValue::Int(value.into()));
        self
    }

    // Nested fields use underscores
    pub fn suggest_address_street(mut self, value: impl Into<String>) -> Self {
        self.suggestions.insert("address.street", ResponseValue::String(value.into()));
        self
    }

    pub fn suggest_address_city(mut self, value: impl Into<String>) -> Self {
        self.suggestions.insert("address.city", ResponseValue::String(value.into()));
        self
    }

    // === Assumption methods ===

    pub fn assume_name(mut self, value: impl Into<String>) -> Self {
        self.assumptions.insert("name", ResponseValue::String(value.into()));
        self
    }

    pub fn assume_age(mut self, value: u32) -> Self {
        self.assumptions.insert("age", ResponseValue::Int(value.into()));
        self
    }

    pub fn assume_address_street(mut self, value: impl Into<String>) -> Self {
        self.assumptions.insert("address.street", ResponseValue::String(value.into()));
        self
    }

    pub fn assume_address_city(mut self, value: impl Into<String>) -> Self {
        self.assumptions.insert("address.city", ResponseValue::String(value.into()));
        self
    }

    // === Bulk methods ===

    /// Pre-fill all fields from an existing instance
    pub fn with_suggestions(mut self, instance: &UserProfile) -> Self {
        self.suggest_name(&instance.name)
            .suggest_age(instance.age)
            .suggest_address_street(&instance.address.street)
            .suggest_address_city(&instance.address.city)
    }

    /// Run the survey with the given backend
    pub fn run<B: SurveyBackend>(self, backend: B) -> Result<UserProfile, SurveyError> {
        // ... apply suggestions/assumptions, run backend, reconstruct
    }
}
```

This approach provides:

1. **Type safety** — `suggest_age` takes `u32`, not a generic value
2. **IDE autocomplete** — type `builder.suggest_` and see all available fields
3. **Compile-time errors** — typos like `suggest_naem` won't compile
4. **Nested field support** — `suggest_address_street` for `address.street`

### Implementation Details

When running the survey, the builder applies suggestions and assumptions:

```rust
impl UserProfileBuilder {
    pub fn run<B: SurveyBackend>(self, backend: B) -> Result<UserProfile, SurveyError> {
        let mut definition = UserProfile::survey();

        // Apply suggestions to question defaults
        apply_suggestions_to_questions(&mut definition.questions, &self.suggestions);

        // Apply assumptions and collect pre-filled responses
        let mut responses = Responses::new();
        apply_assumptions_to_questions(&mut definition.questions, &self.assumptions, &mut responses);

        // Run the survey (only non-assumed questions)
        let collected = backend.collect(&definition, &UserProfile::validate_field)
            .map_err(|e| SurveyError::Backend(e.into()))?;
        responses.extend(collected);

        Ok(UserProfile::from_responses(&responses))
    }
}

fn apply_suggestions_to_questions(
    questions: &mut [Question],
    suggestions: &HashMap<&'static str, ResponseValue>,
) {
    for question in questions {
        if let Some(value) = suggestions.get(question.path().to_string().as_str()) {
            question.set_suggestion(value.clone());
        }
        // Recurse into nested questions
        if let QuestionKind::AllOf(nested) = question.kind_mut() {
            apply_suggestions_to_questions(nested.questions_mut(), suggestions);
        }
    }
}

fn apply_assumptions_to_questions(
    questions: &mut [Question],
    assumptions: &HashMap<&'static str, ResponseValue>,
    responses: &mut Responses,
) {
    for question in questions {
        if let Some(value) = assumptions.get(question.path().to_string().as_str()) {
            question.set_assumption(value.clone());
            responses.insert(question.path().clone(), value.clone());
        }
        // Recurse into nested questions
        if let QuestionKind::AllOf(nested) = question.kind_mut() {
            apply_assumptions_to_questions(nested.questions_mut(), assumptions, responses);
        }
    }
    // Remove assumed questions from the list
    questions.retain(|q| !q.is_assumed());
}
```

### Combining Suggestions and Assumptions

Suggestions and assumptions serve different purposes:

| Feature    | User sees prompt? | User can change? | Use case                                            |
|------------|-------------------|------------------|-----------------------------------------------------|
| Suggestion | Yes               | Yes              | "Here's a sensible default, feel free to change it" |
| Assumption | No                | No               | "We already know this value, no need to ask"        |

They can be combined for different fields:

```rust
let existing = ServerConfig { host: "localhost".into(), port: 8080 };

let config: ServerConfig = ServerConfig::builder()
    .with_suggestions(&existing)  // Pre-fill both fields
    .assume("host", ResponseValue::String("127.0.0.1".into()))  // Override: skip host entirely
    .run(backend)
    .unwrap();

// User only sees:
//   Port: [8080]    <- suggestion from existing instance
// Host is skipped entirely, uses "127.0.0.1"
```

## Nested Surveys

A struct field can itself be a `Survey`-implementing type:

```rust
#[derive(Survey)]
struct Address {
    #[ask("Street:")]
    street: String,
    #[ask("City:")]
    city: String,
}

#[derive(Survey)]
struct Person {
    #[ask("Name:")]
    name: String,
    #[ask("Home address:")]
    address: Address,  // Nested survey
}
```

### How Nested Surveys Work

When the macro encounters a field with a non-builtin type that has `#[ask(...)]`, it recognizes this as a nested survey.

The hierarchy is preserved using `AllOf`. The nested struct's questions become children:

```rust
// Person::survey() generates roughly:
fn survey() -> SurveyDefinition {
    SurveyDefinition {
        questions: vec![
            Question {
                path: ResponsePath::root("name"),
                ask: "Name:",
                kind: QuestionKind::Input(...),
            },
            Question {
                path: ResponsePath::root("address"),
                ask: "Home address:",
                kind: QuestionKind::AllOf(Address::survey().questions),
            },
        ],
        prelude: None,
        epilogue: None,
    }
}
```

The nested `Address` survey may itself have `prelude`/`epilogue` — backends are free to display or ignore them.

Response paths are hierarchical:

- `name`
- `address.street`
- `address.city`

### Nested Validation

Validation is delegated to nested types. When `validate_field` is called with a path starting with `"address"`:

```rust
fn validate_field(path: &ResponsePath, value: &str, responses: &Responses) -> Result<(), String> {
    // Check for nested paths
    if let Some(nested_path) = path.strip_prefix("address") {
        return Address::validate_field(&nested_path, value, responses);
    }

    // Direct field validation
    match path.as_str() {
        "name" => validate_name(value, responses),
        _ => Ok(()),
    }
}
```

### Nested Response Reconstruction

When `from_responses` is called, nested responses are extracted by prefix:

```rust
fn from_responses(responses: &Responses) -> Self {
    // Extract nested responses for Address
    let address_responses = responses.filter_prefix(&ResponsePath::root("address"));
    let address = Address::from_responses(&address_responses);

    Person {
        name: responses.get_string(&ResponsePath::root("name")),
        address,
    }
}
```

### Enum Variants as Nested Surveys

Enums work similarly. Each variant is presented as an alternative:

```rust
#[derive(Survey)]
enum ContactMethod {
    Email { address: String },
    Phone { number: String },
}

#[derive(Survey)]
struct Contact {
    #[ask("Name:")]
    name: String,
    #[ask("Contact method:")]
    method: ContactMethod,
}
```

The user first selects a variant ("Email" or "Phone"), then fills in that variant's fields.

### Enum Variant Selection Storage

When an enum field is collected, the selected variant index is stored using a reserved key `"selected_variant"` within that field's namespace (see [QuestionKind](#questionkind) section for detailed response path examples):

```rust
// For the Contact example above, responses might contain:
// "name"                          -> "Alice"
// "method.selected_variant"       -> ChosenVariant(0)  (Email variant)
// "method.address"                -> "alice@example.com"
```

For a struct with multiple enum fields:

```rust
struct Order {
    #[ask("Payment:")]
    payment: PaymentMethod,
    #[ask("Shipping:")]
    shipping: ShippingMethod,
}

// Responses:
// "payment.selected_variant"      -> ChosenVariant(1)
// "payment.card_number"           -> "1234..."
// "shipping.selected_variant"     -> ChosenVariant(0)
```

Each enum field gets its own prefixed namespace, so variant selections don't collide. When reconstructing the struct, `from_responses` extracts responses by prefix and passes them to each enum's `from_responses`, which reads `"selected_variant"` to determine which variant to construct.

For a **top-level enum** (where the enum itself is the Survey type), the key is simply `"selected_variant"` without a prefix.

### Backend Presentation

Backends decide how to present nested structures:

**Wizard backends:** Flatten everything into a linear sequence. The user answers `name`, then `address.street`, then `address.city`.

**Form backends:** Can render nested surveys as:

- Flat form (all fields at same level)
- Grouped sections (visual grouping by nesting)
- Collapsible panels
- Tabs

The `SurveyDefinition` structure preserves hierarchy information, so form backends can render nested surveys however they prefer.

### Optional Fields

Fields of type `Option<T>` are always presented to the user. An empty response becomes `None`:

```rust
#[derive(Survey)]
struct Person {
    #[ask("Full name:")]
    name: String,

    #[ask("Middle name (optional):")]
    middle_name: Option<String>,

    #[ask("Age:")]
    age: Option<u32>,
}
```

The user is always prompted for optional fields. If they provide an empty string (or skip the field in a form), the value becomes `None`. If they provide a value, it becomes `Some(value)`.

To skip an optional field entirely (not prompt the user at all), use an assumption:

```rust
let person = Person::builder()
    .assume("middle_name", "")  // Skip prompt, value will be None
    .run(backend)
    .unwrap();
```

## Error Handling

The error model is intentionally minimal. Most "errors" are handled internally by backends, not surfaced to callers.

### Error Categories

| Error Category          | Handling                  | Visible to Caller? |
|-------------------------|---------------------------|--------------------|
| Validation              | Backend retry loop        | No                 |
| Response reconstruction | Ruled out by construction | No                 |
| Cancellation            | User exits early          | Yes                |
| Backend failure         | I/O, UI crash             | Yes                |

### Validation Errors — Internal to Backends

Validation failures are not errors in the API sense. They're part of the normal survey flow:

1. User enters invalid input
1. Backend shows error message
1. User corrects input
1. Loop until valid or cancelled

The backend handles this internally. The caller never sees validation errors — they either get a valid `T` or a cancellation/backend failure.

### Response Errors — Ruled Out by Construction

If the macro generates both `survey()` and `from_responses()`, they're guaranteed to be consistent. The macro knows exactly what paths exist and what types they have. If a backend collects all required responses, reconstruction cannot fail.

This is enforced by:

- The macro generates both sides (questions and reconstruction)
- Backends must answer all non-assumed questions before returning
- Type conversions are validated during collection, not reconstruction

There's no response error type — reconstruction failure would be a bug in the macro, not a runtime error.

### Macro Errors — Compile Time Only

Invalid macro annotations are caught at compile time:

```rust
// Generated by macro for invalid attribute
compile_error!("Invalid #[validate] attribute: function `nonexistent` not found");
```

These never reach runtime, so they don't need runtime error types.

### SurveyError — The Public Error Type

```rust
pub enum SurveyError {
    /// User cancelled the survey (Ctrl+C, closed window, etc.)
    Cancelled,

    /// Backend-specific failure (I/O, UI framework crash, etc.)
    Backend(anyhow::Error),
}
```

Only two variants:

- **Cancelled**: User chose to exit without completing
- **Backend**: Something went wrong in the backend (I/O error, terminal lost, GUI crashed, etc.)

### Backend Trait

Backends implement the `SurveyBackend` trait (see [SurveyBackend Trait](#surveybackend-trait) section for full definition).

The key point for error handling: backends return `Result<Responses, Self::Error>` where `Self::Error: Into<anyhow::Error>`. This permissive bound accepts:

- Custom error types implementing `std::error::Error` ✓
- `anyhow::Error` itself ✓
- `std::io::Error` ✓
- Any `thiserror` derived error ✓

Backends handle validation internally in retry loops — errors are only returned for cancellation or backend failures.

### Builder Conversion

The full implementation in the Suggestions and Assumptions section shows the complete flow. Note: `from_responses` returns `T`, not `Result<T, _>`. The macro guarantees it works.

## Summary

The key insight: **the derive macro is presentation-agnostic**. It generates a `SurveyDefinition` data structure. What consumers do with that structure is up to them:

- **Wizard backends** iterate through questions sequentially, return `Responses`
- **Form backends** render all questions at once, return `Responses`
- **Output crates** (like typst) generate documents, don't return `Responses`

The crate names communicate purpose:

- `derive-{library}-wizard` / `derive-{library}-form` — interactive backends
- `derive-{library}-document` — output formats

All share the same `derive-survey` foundation.
