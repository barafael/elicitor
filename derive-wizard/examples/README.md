# Derive Wizard Examples

This directory contains examples demonstrating various features and use cases of the Derive Wizard library.

## Directory Structure

### 📁 basic/
Fundamental examples showing core functionality:
- **simple_struct.rs** - Basic struct with primitive types (String, i32, bool)
- **enum_variants.rs** - Enum with multiple variants and associated data
- **nested_structs.rs** - Nested struct composition
- **pathbuf.rs** - Using PathBuf types

### 📁 features/
Examples demonstrating specific features:
- **validation.rs** - Input validation with `#[validate]`, `#[validate_on_key]`, and `#[validate_on_submit]`
- **password_masking.rs** - Password masking with `#[mask]` attribute
- **editor.rs** - Multi-line text input with `#[editor]` attribute
- **suggestions.rs** - Pre-filling values with suggestions using `.with_suggestions()`
- **assumptions.rs** - Using `.assume_field()` to skip questions entirely

### 📁 backends/
Backend-specific examples:

**Dialoguer Backend:**
- **dialoguer.rs** - Basic dialoguer backend usage
- **dialoguer_password.rs** - Password masking in dialoguer
- **dialoguer_suggestions.rs** - Suggestions display in dialoguer

**Egui Backend:**
- **egui.rs** - Basic egui GUI backend usage
- **egui_suggestions.rs** - Suggestions in egui
- **egui_assumptions_and_suggestions.rs** - Comprehensive demo of assumptions vs suggestions

### 📁 builders/
Builder API examples:
- **builder_api.rs** - Basic builder pattern usage
- **builder_comprehensive.rs** - Comprehensive builder API demonstration
- **builder_egui.rs** - Builder API with egui backend

### 📁 advanced/
Complex use cases:
- **deeply_nested.rs** - Multiple levels of nested structs
- **nested_enum_payment.rs** - Struct containing an enum field

### 📄 showcase.rs
Comprehensive showcase demonstrating all major field types and attributes in one example.
- Supports both requestty (default) and egui backends
- Run with `--features egui-backend` to use the GUI version
- Demonstrates: String, password, multiline editor, bool, i32, f32, f64, and enum fields

## Running Examples

To run any example, use `cargo run --example <name>`:

```bash
# Basic examples
cargo run --example simple_struct
cargo run --example enum_variants
cargo run --example nested_structs

# Feature examples
cargo run --example validation
cargo run --example password_masking
cargo run --example suggestions

# Backend examples (require feature flags)
cargo run --example dialoguer --features dialoguer-backend
cargo run --example egui --features egui-backend

# Advanced examples
cargo run --example deeply_nested
cargo run --example nested_enum_payment

# Comprehensive showcase
cargo run --example showcase                      # Uses requestty (default)
cargo run --example showcase --features egui-backend  # Uses egui GUI
```

## Key Concepts

### Suggestions vs Assumptions

**Suggestions** (`.with_suggestions()`):
- Pre-fill form fields with default values
- User is still prompted for all fields
- User can accept defaults or enter new values
- Use case: Editing existing configurations

**Assumptions** (`.assume_field()`):
- Skip questions entirely for specified fields
- User is NOT prompted for those fields
- Values are fixed and cannot be changed during the wizard
- Use case: Automation, security policies, fixed templates

### Backends

The library supports multiple backends:
- **Requestty** (default) - Terminal UI
- **Dialoguer** - Simple terminal prompts
- **Egui** - GUI interface

Use `.with_backend()` to specify a backend:
```rust
let backend = derive_wizard::DialoguerBackend::new();
let config = MyStruct::wizard_builder()
    .with_backend(backend)
    .build();
```

### Validation

Three types of validation:
- `#[validate("fn_name")]` - General validation
- `#[validate_on_key("fn_name")]` - Real-time validation as user types
- `#[validate_on_submit("fn_name")]` - Validate only on submission

Validation functions have signature:
```rust
fn validate(input: &str, answers: &Answers) -> Result<(), String>
```
