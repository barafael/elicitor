# Implementation Plan: derive-wizard → derive-survey

This plan transforms the current `derive-wizard` crate into the new `derive-survey` architecture as described in [new-architecture.md](new-architecture.md).

## Overview of Changes

### Current State → Target State

| Current | Target |
|---------|--------|
| `derive-wizard` | `derive-survey` |
| `derive-wizard-types` | `derive-survey-types` |
| `derive-wizard-macro` | `derive-survey-macro` |
| `#[derive(Wizard)]` | `#[derive(Survey)]` |
| `#[prompt("...")]` | `#[ask("...")]` |
| `Interview` | `SurveyDefinition` |
| `Question` with `id`/`name`/`prompt` | `Question` with `path`/`ask` |
| `QuestionKind::Sequence` | `QuestionKind::AllOf` |
| `QuestionKind::Alternative` | `QuestionKind::OneOf` with `Variant` |
| `QuestionKind::MultiSelect` | `QuestionKind::AnyOf` |
| `Answers` (string keys) | `Responses` (`ResponsePath` keys) |
| `InterviewBackend` | `SurveyBackend` |
| `WizardBuilder` | `SurveyBuilder` |
| Backends in main crate | Backends in separate crates |

---

## Phase 1: Rename and Restructure Types (derive-survey-types)

### Step 1.1: Rename crate

- [ ] Rename `derive-wizard-types/` → `derive-survey-types/`
- [ ] Update `Cargo.toml`: name, description

### Step 1.2: Create `ResponsePath` type

- [ ] Create `src/response_path.rs` with:
  - `ResponsePath { segments: Vec<String> }`
  - `root()`, `child()`, `segments()`, `strip_prefix()`, `as_str()`
  - Implement `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`
  - Implement `From<&str>`, `From<String>`

### Step 1.3: Create `ResponseValue` enum

- [ ] Create `src/response_value.rs` with:
  - `String(String)`, `Int(i64)`, `Float(f64)`, `Bool(bool)`
  - `ChosenVariant(usize)` — for OneOf selection
  - `ChosenVariants(Vec<usize>)` — for AnyOf selection
- [ ] Remove `Nested(Box<Answers>)` — flat structure with paths instead

### Step 1.4: Create `Responses` type

- [ ] Create `src/responses.rs` with:
  - `Responses { values: HashMap<ResponsePath, ResponseValue> }`
  - `get()`, `insert()`, `filter_prefix()`
  - Convenience: `get_string()`, `get_int()`, `get_float()`, `get_bool()`, `get_chosen_variant()`, `get_chosen_variants()`

### Step 1.5: Create `DefaultValue` enum

- [ ] Add to types:
  - `None`, `Suggested(ResponseValue)`, `Assumed(ResponseValue)`

### Step 1.6: Refactor `Question` struct

- [ ] Replace `id: Option<String>` + `name: String` + `prompt: String` with:
  - `path: ResponsePath`
  - `ask: String`
- [ ] Replace `assumed: Option<AssumedAnswer>` with `default: DefaultValue`
- [ ] Add `set_suggestion()` and `set_assumption()` methods

### Step 1.7: Refactor `QuestionKind` enum

- [ ] Rename `Sequence` → `AllOf(Vec<Question>)`
- [ ] Replace `Alternative(usize, Vec<Question>)` with `OneOf(Vec<Variant>)`
- [ ] Replace `MultiSelect(MultiSelectQuestion)` with `AnyOf(AnyOfQuestion)`
- [ ] Add `None` variant for unit types
- [ ] Keep: `Input`, `Multiline`, `Masked`, `Int`, `Float`, `Confirm`

### Step 1.8: Create `Variant` struct

- [ ] Create struct: `Variant { name: String, kind: QuestionKind }`

### Step 1.9: Create `AnyOfQuestion` struct

- [ ] Create struct: `AnyOfQuestion { variants: Vec<Variant>, defaults: Vec<usize> }`

### Step 1.10: Rename `Interview` → `SurveyDefinition`

- [ ] Change struct name
- [ ] Rename `sections` → `questions`
- [ ] Keep `prelude` and `epilogue`

### Step 1.11: Create `SurveyError` enum

- [ ] Create `src/error.rs` with:
  - `Cancelled`
  - `Backend(anyhow::Error)`
- [ ] Add `anyhow` dependency

### Step 1.12: Define traits

- [ ] Create `Survey` trait in `src/survey.rs`:
  - `fn survey() -> SurveyDefinition`
  - `fn from_responses(responses: &Responses) -> Self`
  - `fn validate_field(path: &ResponsePath, responses: &Responses) -> Result<(), String>`
  - `fn validate_all(responses: &Responses) -> HashMap<ResponsePath, String>`
  - `fn builder() -> SurveyBuilder<Self>`
- [ ] Create `SurveyBackend` trait:
  - `type Error: Into<anyhow::Error>`
  - `fn collect(&self, definition: &SurveyDefinition, validate: &dyn Fn(&ResponsePath, &Responses) -> Result<(), String>) -> Result<Responses, Self::Error>`

### Step 1.13: Clean up old types

- [ ] Remove `SuggestedAnswer` (replaced by `DefaultValue::Suggested`)
- [ ] Remove `AssumedAnswer` (replaced by `DefaultValue::Assumed`)
- [ ] Remove `SELECTED_ALTERNATIVE_KEY` constant (use reserved path pattern)
- [ ] Update `lib.rs` exports

---

## Phase 2: Update Main Crate (derive-survey)

### Step 2.1: Rename crate

- [ ] Rename `derive-wizard/` → `derive-survey/`
- [ ] Update `Cargo.toml`: name, description, dependencies

### Step 2.2: Remove backend implementations from main crate

- [ ] Delete `src/backend/requestty_backend.rs`
- [ ] Delete `src/backend/dialoguer_backend.rs`
- [ ] Delete `src/backend/egui_backend.rs`
- [ ] Delete `src/backend/ratatui_backend.rs`
- [ ] Remove backend feature flags from `Cargo.toml`
- [ ] Remove backend dependencies (requestty, egui, eframe, dialoguer, ratatui, crossterm)

### Step 2.3: Create `SurveyBuilder`

- [ ] Create `src/builder.rs` with:
  - `SurveyBuilder<T: Survey>`
  - `with_suggestions(&T)` — pre-fill from instance
  - `suggest(path, value)` — individual suggestion
  - `suggest_all(...)` — batch suggestions
  - `assume(path, value)` — skip question with value
  - `assume_all(...)` — batch assumptions
  - `run<B: SurveyBackend>(backend) -> Result<T, SurveyError>`

### Step 2.4: Keep `TestBackend`

- [ ] Update to use new types (`Responses`, `ResponsePath`, etc.)
- [ ] Make it private (not exported)
- [ ] Update `with_response()` method signature

### Step 2.5: Update `lib.rs`

- [ ] Re-export all types from `derive-survey-types`
- [ ] Re-export `#[derive(Survey)]` from `derive-survey-macro`
- [ ] Export `SurveyBuilder`
- [ ] Remove compile_error for missing backends

### Step 2.6: Remove/update other modules

- [ ] Delete or update `src/answer.rs` (replaced by `Responses` in types)
- [ ] Delete or update `src/field_path.rs` (replaced by `ResponsePath` in types)
- [ ] Update `src/typst_form.rs` to use new types (or move to separate crate later)

---

## Phase 3: Update Proc-Macro (derive-survey-macro)

### Step 3.1: Rename crate

- [ ] Rename `derive-wizard-macro/` → `derive-survey-macro/`
- [ ] Update `Cargo.toml`

### Step 3.2: Rename derive macro

- [ ] `#[proc_macro_derive(Wizard, ...)]` → `#[proc_macro_derive(Survey, ...)]`

### Step 3.3: Update attribute names

- [ ] `#[prompt("...")]` → `#[ask("...")]`
- [ ] Keep: `#[mask]`, `#[multiline]`, `#[validate]`, `#[min]`, `#[max]`, `#[prelude]`, `#[epilogue]`
- [ ] Add: `#[multiselect]` for `Vec<Enum>` fields

### Step 3.4: Update generated trait implementation

- [ ] Generate `Survey` trait instead of `Wizard` trait
- [ ] Update method signatures:
  - `fn survey()` → `fn survey()`
  - `fn interview_with_suggestions(&self)` → remove (handled by builder)
  - `fn from_answers()` → `fn from_responses()`
  - `fn validate_field()` — update to use `ResponsePath`
- [ ] Add `fn validate_all()` generation

### Step 3.5: Update code generation for `survey()`

- [ ] Generate `SurveyDefinition` instead of `Interview`
- [ ] Use `ResponsePath` for question paths
- [ ] Generate `AllOf` for struct fields and struct variants
- [ ] Generate `OneOf` with `Variant` for enums
- [ ] Generate `AnyOf` for `Vec<Enum>` fields
- [ ] Store variant selection as `"selected_variant"` path suffix

### Step 3.6: Update code generation for `from_responses()`

- [ ] Use `Responses` instead of `Answers`
- [ ] Use `ResponsePath` for lookups
- [ ] Handle `ChosenVariant` for OneOf reconstruction
- [ ] Handle `ChosenVariants` for AnyOf reconstruction
- [ ] Use `filter_prefix()` for nested type reconstruction

### Step 3.7: Update validation code generation

- [ ] Generate `validate_field()` using `ResponsePath`
- [ ] Generate `validate_all()` for composite validators
- [ ] Keep compile-time validator signature checks

---

## Phase 4: Create Separate Backend Crates (Future)

> Note: This phase can be done incrementally after the core refactoring.

### Step 4.1: Create `derive-requestty-wizard`

- [ ] New crate with `RequesttyWizard` implementing `SurveyBackend`
- [ ] Move logic from current `requestty_backend.rs`
- [ ] Depends on `derive-survey` and `requestty`

### Step 4.2: Create `derive-dialoguer-wizard`

- [ ] Similar structure for dialoguer backend

### Step 4.3: Create `derive-ratatui-wizard`

- [ ] Wizard-style ratatui backend

### Step 4.4: Create `derive-ratatui-form` (future)

- [ ] Form-style ratatui backend

### Step 4.5: Create `derive-egui-form`

- [ ] Form-style egui backend

### Step 4.6: Create `derive-typst-document` (future)

- [ ] Output crate for Typst form generation

---

## Phase 5: Update Examples and Tests

### Step 5.1: Update all examples

- [ ] Replace `#[derive(Wizard)]` → `#[derive(Survey)]`
- [ ] Replace `#[prompt("...")]` → `#[ask("...")]`
- [ ] Update backend usage

### Step 5.2: Update tests

- [ ] Update `wizard_tests.rs` → `survey_tests.rs`
- [ ] Use new type names and APIs

### Step 5.3: Update documentation

- [ ] Update README files
- [ ] Update doc comments

---

## Phase 6: Workspace and CI Updates

### Step 6.1: Update workspace Cargo.toml

- [ ] Update member paths to new crate names

### Step 6.2: Update example-wizard

- [ ] Rename to `example-survey`
- [ ] Update dependencies and code

---

## Implementation Order (Recommended)

1. **Phase 1** (types) — Foundation, everything depends on this
2. **Phase 3** (macro) — Update code generation for new types
3. **Phase 2** (main crate) — Wire everything together
4. **Phase 5** (examples/tests) — Verify it all works
5. **Phase 6** (workspace) — Final cleanup
6. **Phase 4** (backend crates) — Can be done later, incrementally

---

## Migration Checklist

- [ ] All crates renamed
- [ ] All types renamed per architecture doc
- [ ] `ResponsePath` used instead of string keys
- [ ] `OneOf`/`AllOf`/`AnyOf` structure for nested types
- [ ] Backends removed from main crate
- [ ] `SurveyBuilder` with suggestions/assumptions working
- [ ] All examples compile and run
- [ ] All tests pass

---

## Notes

- **No backwards compatibility required** — clean break
- Focus on core types first, then macro, then integration
- Backend extraction (Phase 4) can happen after core is stable
- Keep `TestBackend` in main crate for testing
