# derive-wizard

A Rust procedural macro that automatically generates interactive CLI wizards from struct definitions using [requestty](https://crates.io/crates/requestty).

## Showcase

```rust
use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
struct ShowCase {
    // String types - defaults to 'input'
    #[prompt("Enter your name:")]
    name: String,

    // Override with password question type
    #[prompt("Enter your password:")]
    #[mask]
    password: String,

    // Long text with editor
    #[prompt("Enter a bio:")]
    #[editor]
    bio: String,

    // Bool type - defaults to 'confirm'
    #[prompt("Do you agree to the terms?")]
    agree: bool,

    // Integer types - defaults to 'int'
    #[prompt("Enter your age (i32):")]
    age: i32,

    // Float types - defaults to 'float'
    #[prompt("Enter your height in meters (f64):")]
    height: f64,

    #[prompt("Enter a decimal number (f32):")]
    decimal: f32,
    
    #[prompt("Enter your gender")]
    gender: Gender,
}

#[derive(Debug, Wizard)]
enum Gender {
    Male,
    Female,
    Other(
        #[prompt("Please specify:")]
        String
    ),
}

```

### Password Fields with `#[mask]`

For password inputs, use the convenient `#[mask]` attribute to hide user input:

```rust
use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
struct LoginForm {
    #[prompt("Enter your username:")]
    username: String,

    #[prompt("Enter your password:")]
    #[mask]
    password: String,  // Input will be hidden
}
```

### Long Text with `#[editor]`

For longer text input, use the `#[editor]` attribute to open the user's preferred text editor:

```rust
use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
struct Article {
    #[prompt("Enter the title:")]
    title: String,

    #[prompt("Write the article content:")]
    #[editor]
    content: String,  // Opens text editor (vim, nano, etc.)
}
```

## Attributes

- `#[prompt("message")]` - **Required**. The message to display to the user
- `#[mask]` - **Optional**. For String fields: enables password input (hidden text)
- `#[editor]` - **Optional**. For String fields: opens text editor for longer input
- `#[validate_on_submit("function_name")]` - **Optional**. Validates input when user submits
- `#[validate_on_key("function_name")]` - **Optional**. Validates input on every keystroke

**Note**: `#[mask]` and `#[editor]` are mutually exclusive and cannot be used on the same field.

## Using the Builder API

The builder API provides a fluent interface for configuring and executing wizards:

```rust,no_run
use derive_wizard::Wizard;

#[derive(Debug, Clone, Wizard)]
struct Config {
    #[prompt("Enter the server address:")]
    server: String,

    #[prompt("Enter the port:")]
    port: u16,

    #[prompt("Enable SSL?")]
    use_ssl: bool,
}

// Simple usage with default backend (requestty)
let config = Config::wizard_builder().build();
println!("Config: {config:#?}");

// Edit configuration with defaults pre-filled
let updated_config = Config::wizard_builder()
    .with_defaults(config)
    .build();
println!("Updated config: {updated_config:#?}");
```

Additional examples:

```rust,no_run
use derive_wizard::Wizard;

# #[derive(Debug, Clone, Wizard)]
# struct Config {
#     #[prompt("Enter the server address:")]
#     server: String,
#     #[prompt("Enter the port:")]
#     port: u16,
#     #[prompt("Enable SSL?")]
#     use_ssl: bool,
# }
// With custom backend (e.g., requestty)
let backend = derive_wizard::RequesttyBackend::new();
let config = Config::wizard_builder()
    .with_backend(backend)
    .build();
println!("Config: {config:#?}");

// Combine defaults with custom backend
let backend = derive_wizard::RequesttyBackend::new();
let updated_config = Config::wizard_builder()
    .with_defaults(config)
    .with_backend(backend)
    .build();
println!("Updated config: {updated_config:#?}");
```

When `with_defaults()` is used:

- For **String** fields: the current value is shown as a hint/placeholder
- For **numeric** fields (integers and floats): the current value is shown as default
- For **bool** fields: the current value is pre-selected
- For **password** (`#[mask]`) and **editor** (`#[editor]`) fields: defaults are shown as hints (backend-dependent)

## Supported Question Types

The `#[derive(Wizard)]` macro supports all 11 requestty question types:

| Rust Type                          | Default Question Type | Override Options                                    | Returns              |
|------------------------------------|-----------------------|-----------------------------------------------------|----------------------|
| `String`                           | `input`               | `#[mask]` for password, `#[editor]` for text editor | `String`             |
| `bool`                             | `confirm`             | -                                                   | `bool`               |
| `i8`, `i16`, `i32`, `i64`, `isize` | `int`                 | -                                                   | `i64` (cast to type) |
| `u8`, `u16`, `u32`, `u64`, `usize` | `int`                 | -                                                   | `i64` (cast to type) |
| `f32`, `f64`                       | `float`               | -                                                   | `f64` (cast to type) |
| `ListItem`                         | `select`              | -                                                   | `ListItem`           |
| `ExpandItem`                       | `expand`              | -                                                   | `ExpandItem`         |
| `Vec<ListItem>`                    | `multi_select`        | -                                                   | `Vec<ListItem>`      |

## Question Type Details

1. **input** - Basic text input prompt (default for String)
2. **password** - Hidden text input (use `#[mask]` on String fields)
3. **editor** - Opens text editor for longer input (use `#[editor]` on String fields)
4. **confirm** - Yes/No confirmation prompt (default for bool)
5. **int** - Integer input (default for integer types)
6. **float** - Floating point input (default for float types)
7. **select** - Single selection from a list (default for ListItem)
8. **expand** - Single selection with keyboard shortcuts (default for ExpandItem)
9. **multi_select** - Multiple selection from a list (default for `Vec<ListItem>`)

Note: The following question types are available in requestty but not currently exposed through attributes:

- **raw_select** - Single selection with index-based input
- **order_select** - Reorder items in a list

## License

MIT OR Apache-2.0
