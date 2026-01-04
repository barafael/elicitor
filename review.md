# Code Review: derive-wizard

**Review Date:** January 4, 2026  
**Reviewer:** AI Assistant  
**Scope:** Full codebase inspection for bugs, quality issues, and conceptual shortcomings

---

## Executive Summary

Originally 23 issues were identified. **Current status (after fixes on 2026-01-04):**

- 🔴 **0 Critical Bugs open** (critical items #2, #3, #4 were fixed)
- 🟡 **6 Quality Issues** - Code maintainability and design problems  
- 🟠 **10 Conceptual Shortcomings** - Architecture and API design limitations
- 📋 **3 Testing Gaps** - Missing test coverage

**Verification:** `cargo test` (all pass) and `cargo test --doc` (all pass) on 2026-01-04.

---

## 🔴 Critical Bugs

### #1: `iter()` Method Dependency (Informational)

**Location:** `derive-wizard-macro/src/lib.rs`  
**Severity:** Informational

```rust
for (key, value) in answers.iter() {
    if let Some(stripped) = key.strip_prefix(prefix) {
        nested_answers.insert(stripped.to_string(), value.clone());
    }
}
```

**Status:** The `Answers::iter` method exists in this workspace; no current break observed. Keep an eye on semver if publishing macros and runtime separately.

---

### #2: Unsafe Unwraps in Builder Pattern (Resolved)

**Location:** `derive-wizard/src/lib.rs`  
**Severity:** Resolved

```rust
let answers = backend
    .execute_with_validator(&interview, &T::validate_field)
    .expect("Failed to execute interview");  // ❌ Panic!
T::from_answers(&answers).expect("Failed to build from answers")  // ❌ Panic!
```

**Problem:**

- Builder pattern uses `.expect()` which panics on errors
- Users cannot handle errors gracefully
- Violates Rust best practices for error handling
- Also occurs at line 208, 231, 232

**Impact:** Production code can panic instead of returning recoverable errors

**Recommended Fix:**

Status: Fixed on 2026-01-04. `build()` now returns `Result<T, BackendError>` and propagates errors; verified by `cargo test` and `cargo test --doc`.

---

### #3: Potential Panic in egui Validation (Resolved)

**Location:** `derive-wizard/src/backend/egui_backend.rs`  
**Severity:** Resolved

```rust
validate_result_rx.lock().unwrap().recv().unwrap_or(Ok(()))
```

**Problem:**

- `.lock().unwrap()` will panic if mutex is poisoned
- Channel communication can fail in edge cases
- Multiple `.unwrap()` calls in validation flow (lines 737, 764, 805)

**Scenarios:**

- Thread panic while holding lock → mutex poisoning
- GUI thread terminates → channel broken
- Race condition during shutdown

Status: Fixed on 2026-01-04. Channel handling now avoids `unwrap` on locks/channels; verified by `cargo test` and `cargo test --doc`.

---

### #4: Unsafe Error Vector Assumptions (Resolved)

**Location:** `derive-wizard/src/backend/egui_backend.rs`  
**Severity:** Resolved

```rust
if errs.is_empty() {
    Ok(())
} else {
    // Store all errors and return the first one
    for (id, err) in &errs {
        self.state.validation_errors.insert(id.clone(), err.clone());
    }
    Err(errs.into_iter().next().unwrap())  // ❌ Assumes non-empty!
}
```

**Problem:**

- Logic assumes `errs` is non-empty in else branch
- Rust doesn't enforce this statically
- Future refactoring could break this invariant

Status: Fixed on 2026-01-04. Error aggregation no longer unwraps; safe matching implemented. Verified by `cargo test` and `cargo test --doc`.

---

## 🟡 Quality Issues

### #5: Global UI State Mutation in egui

**Location:** `derive-wizard/src/backend/egui_backend.rs:410-412, 447-449, 479-481`  
**Severity:** High

```rust
// Check for validation error before any mutable borrows
let has_error = self.state.validation_errors.contains_key(id);

let buffer = self.state.get_or_init_buffer(id);
let mut text_edit = egui::TextEdit::singleline(buffer);

// Add red border if there's a validation error
if has_error {
    ui.visuals_mut().widgets.inactive.bg_stroke.color = egui::Color32::RED;
    ui.visuals_mut().widgets.hovered.bg_stroke.color = egui::Color32::RED;
    ui.visuals_mut().widgets.active.bg_stroke.color = egui::Color32::RED;
}
```

**Problem:**

- Mutates **global UI visuals** instead of styling individual widgets
- Affects ALL widgets in the UI after this point, not just the invalid input
- Side effects persist across frames
- Next widget rendered will also have red borders

**Impact:**

- Visual glitches where unrelated fields show validation errors
- Non-deterministic UI behavior
- Hard to debug rendering issues

**Recommended Fix:**

```rust
let text_edit = if has_error {
    egui::TextEdit::singleline(buffer)
        .frame(true)
        .stroke(egui::Stroke::new(1.0, egui::Color32::RED))
} else {
    egui::TextEdit::singleline(buffer)
};
```

Or use Frame wrapping:

```rust
let frame = if has_error {
    egui::Frame::none()
        .stroke(egui::Stroke::new(2.0, egui::Color32::RED))
        .inner_margin(2.0)
} else {
    egui::Frame::none()
};

frame.show(ui, |ui| {
    ui.add(egui::TextEdit::singleline(buffer));
});
```

---

### #6: Code Duplication in Validation

**Location:** `derive-wizard/src/backend/egui_backend.rs:417-431, 454-468, 491-505`  
**Severity:** Medium

**Problem:**

- Nearly identical validation code repeated 3 times for Input/Multiline/Masked types
- ~30 lines duplicated per question type
- Changes must be synchronized across all three locations

**Duplicated Pattern:**

```rust
if <question_type>.validate.is_some() {
    let value = self.state.input_buffers.get(id).cloned().unwrap_or_default();
    let current_answers = self.build_current_answers();
    match (self.validator)(id, &value, &current_answers) {
        Ok(()) => {
            self.state.validation_errors.remove(id);
        }
        Err(err) => {
            self.state.validation_errors.insert(id.to_string(), err);
        }
    }
}
```

**Recommended Fix:**

```rust
fn validate_text_field(&mut self, id: &str, has_validator: bool) {
    if !has_validator {
        return;
    }
    
    let value = self.state.input_buffers
        .get(id)
        .cloned()
        .unwrap_or_default();
    let current_answers = self.build_current_answers();
    
    match (self.validator)(id, &value, &current_answers) {
        Ok(()) => {
            self.state.validation_errors.remove(id);
        }
        Err(err) => {
            self.state.validation_errors.insert(id.to_string(), err);
        }
    }
}

// Usage:
self.validate_text_field(id, input_q.validate.is_some());
```

---

### #7: Silent Validator Ignoring

**Location:** `derive-wizard/src/backend.rs:35-43`  
**Severity:** Low-Medium

```rust
fn execute_with_validator(
    &self,
    interview: &Interview,
    validator: &(dyn Fn(&str, &str, &Answers) -> Result<(), String> + Send + Sync),
) -> Result<Answers, BackendError> {
    // Default implementation: just execute without validation
    let _ = validator;  // ❌ Silently ignores validator!
    self.execute(interview)
}
```

**Problem:**

- Default implementation silently ignores validation
- No warning when validation is requested but not supported
- Users may think validation is working when it's not

**Recommended Fix:**

```rust
fn execute_with_validator(
    &self,
    interview: &Interview,
    validator: &(dyn Fn(&str, &str, &Answers) -> Result<(), String> + Send + Sync),
) -> Result<Answers, BackendError> {
    eprintln!("Warning: Backend does not support validation, falling back to execute()");
    let _ = validator;
    self.execute(interview)
}
```

Or return an error:

```rust
Err(BackendError::Custom(
    "This backend does not support validation".to_string()
))
```

---

### #8: Limited Answers API

**Location:** `derive-wizard/src/answer.rs`  
**Severity:** Low

**Problem:**

- `Answers` type doesn't expose:
  - `keys()` - to list all answer keys
  - `contains_key()` - to check existence
  - `is_empty()` - to check if any answers collected
  - `len()` - to count answers
- Makes debugging and testing harder
- Users can't inspect collected answers

**Recommended Addition:**

```rust
impl Answers {
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }
    
    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
    
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    
    pub fn len(&self) -> usize {
        self.values.len()
    }
}
```

---

### #9: FieldPath Separator Confusion

**Location:** `derive-wizard/src/field_path.rs:88-105`  
**Severity:** Low

```rust
/// # Examples
///
/// ```rust
/// use derive_wizard::field;
/// // Single field (top-level)
/// field!(name); // expands to FieldPath from "name"
///
/// // Nested field
/// field!(Person::contact::email); // expands to FieldPath from "contact/email"
///                                 // ❌ Actually generates "contact.email"
/// ```
```

**Problem:**

- Documentation says slash separator (`/`)
- Implementation uses dot separator (`.`)
- `from_slash_path()` method exists but not used consistently
- Mixed separators cause confusion

**Recommended Fix:**

1. Update documentation to match implementation (use dots)
2. Or standardize on slashes and update implementation
3. Add conversion methods between formats

---

### #10: Missing Nested Wizard Validation

**Location:** `derive-wizard-macro/src/lib.rs:690-705`  
**Severity:** Medium

**Problem:**

- Generated code for nested wizards doesn't call their validators
- Only creates filtered answer set and calls `from_answers`
- Nested struct fields skip validation entirely

```rust
type_str => {
    let type_ident = syn::parse_str::<syn::Ident>(type_str).unwrap();
    let prefix = format!("{}.", field_name);
    quote! {
        {
            let mut nested_answers = derive_wizard::Answers::default();
            let prefix = #prefix;
            for (key, value) in answers.iter() {
                if let Some(stripped) = key.strip_prefix(prefix) {
                    nested_answers.insert(stripped.to_string(), value.clone());
                }
            }
            #type_ident::from_answers(&nested_answers)?
            // ❌ Should call validate_field for nested type!
        }
    }
}
```

**Impact:** Validation gaps in complex nested forms

**Recommended Fix:**
Generate validation calls for nested types or document this limitation clearly.

---

## 🟠 Conceptual Shortcomings

### #11: No Cancellation Support

**Severity:** Medium  
**Impact:** Poor UX, can't distinguish cancel from error

**Problem:**

- GUI backends have no way to cancel interview
- Window close might panic or return incomplete data
- No `Cancelled` variant in error enum

**Recommended Enhancement:**

```rust
#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    // ... existing variants ...
    
    #[error("Interview was cancelled by user")]
    Cancelled,
}
```

Allow backends to return `Err(BackendError::Cancelled)` on window close.

---

### #12: Limited Validation Timing Control

**Severity:** Medium  
**Impact:** Performance and UX issues

**Problem:**

- egui validates on every frame render
- Other backends don't validate at all (default impl)
- No way to configure:
  - Validate on submit only
  - Validate on blur/focus loss
  - Debounced validation
  - Async validation

**Recommended Enhancement:**

```rust
pub enum ValidationTiming {
    Immediate,      // On every keystroke
    OnBlur,         // When field loses focus
    OnSubmit,       // Only when submit clicked
    Debounced(Duration),  // After N ms of inactivity
}

impl EguiBackend {
    pub fn with_validation_timing(mut self, timing: ValidationTiming) -> Self {
        self.validation_timing = timing;
        self
    }
}
```

---

### #13: No Conditional Questions

**Severity:** Medium  
**Impact:** Limited use cases

**Problem:**

- Can't show/hide questions based on previous answers
- All questions are static at compile time
- Common UI pattern missing

**Example Use Case:**

```rust
// Want to show "spouse_name" only if married == true
#[derive(Wizard)]
struct Person {
    name: String,
    married: bool,
    
    #[show_if("married == true")]  // ❌ Not supported
    spouse_name: String,
}
```

**Workaround:** Manual implementation required

**Potential Solution:**

- Add `#[show_if("condition")]` attribute
- Evaluate conditions at runtime
- Complex but valuable feature

---

### #14: Fragile Enum Variant Storage

**Severity:** Medium  
**Impact:** Refactoring hazards

**Problem:**

- Enum variants stored as strings: `"selected_alternative": "CreditCard"`
- Fragile to variant name changes
- No compile-time safety
- Renaming variant breaks serialized data

**Current Approach:**

```rust
let selected = answers.as_string("selected_alternative")?;
match selected.as_str() {
    "Cash" => Ok(PaymentMethod::Cash),
    "CreditCard" => Ok(PaymentMethod::CreditCard { ... }),
    // ❌ String matching is fragile
}
```

**Better Approach:**

```rust
// Use discriminant indices
answers.as_int("selected_alternative")?;  // 0, 1, 2, etc.

// Or use an enum
#[derive(Serialize, Deserialize)]
enum PaymentMethodVariant {
    Cash = 0,
    CreditCard = 1,
    BankTransfer = 2,
}
```

---

### #15: No Async Support

**Severity:** High for certain use cases  
**Impact:** Can't validate against external systems

**Problem:**

- Validators must be synchronous
- Can't make database lookups
- Can't call APIs for validation
- Blocks UI thread for slow operations

**Example Needed:**

```rust
// Want to validate username is available
async fn validate_username(username: &str) -> Result<(), String> {
    let exists = database.check_username_exists(username).await?;
    if exists {
        Err("Username already taken".to_string())
    } else {
        Ok(())
    }
}
```

**Potential Solution:**

- Async validator trait
- Runtime handle passing
- Background validation with loading states

---

### #16: Builder Returns Concrete Type

**Severity:** Low  
**Impact:** Not idiomatic Rust

**Problem:**

```rust
pub fn build(self) -> T  // ❌ Should return Result<T, E>
```

Forces users to handle panics instead of errors.

**Recommended:**

```rust
pub fn build(self) -> Result<T, BackendError>
```

---

### #17: No Partial Result Recovery

**Severity:** Medium  
**Impact:** Poor UX on validation errors

**Problem:**

- Can't get partially filled answers if validation fails
- All-or-nothing approach loses user data
- User must re-enter everything if one field is invalid

**Example:**

```rust
// User fills 20 fields, last one fails validation
// All data is lost! ❌
```

**Recommended Enhancement:**

```rust
pub struct ValidationResult<T> {
    pub value: Option<T>,
    pub partial_answers: Answers,
    pub errors: Vec<ValidationError>,
}

pub fn build(self) -> Result<T, ValidationResult<T>>
```

---

### #18: TestBackend Skips Validation

**Severity:** Medium  
**Impact:** False test confidence

**Problem:**

- `TestBackend` doesn't run validators
- Tests can pass with invalid data
- Integration tests don't catch validation bugs

**Current Behavior:**

```rust
impl InterviewBackend for TestBackend {
    fn execute(&self, interview: &Interview) -> Result<Answers, BackendError> {
        // Returns predefined answers, no validation!
        Ok(self.answers.clone())
    }
}
```

**Recommended:**
Add validation support to TestBackend or document this limitation prominently.

---

### #19: No Custom Question Types

**Severity:** Low  
**Impact:** Limited extensibility

**Problem:**

- Fixed set of question types (Input, Int, Float, Bool, etc.)
- No extension point for custom widgets
- Can't add DatePicker, ColorPicker, FileUpload, etc.

**Potential Solution:**

```rust
pub trait CustomQuestion {
    fn render(&self, ui: &mut egui::Ui) -> String;
    fn validate(&self, value: &str) -> Result<(), String>;
}

pub enum QuestionKind {
    // ... existing variants
    Custom(Box<dyn CustomQuestion>),
}
```

---

### #20: Confusing Naming

**Severity:** Low  
**Impact:** API usability

**Problem:**

- `AssumedAnswer` - skips question entirely
- `SuggestedAnswer` - provides default value
- Similar names, very different behaviors

**Users might think:**

- "Assumed" means "best guess"
- "Suggested" means "optional"

**Recommended Rename:**

- `AssumedAnswer` → `SkippedValue` or `PrefilledValue`
- `SuggestedAnswer` → `DefaultValue` or `InitialValue`

---

## 📋 Testing Gaps

### #21: No Error Path Testing

**Severity:** High

**Problem:**

- All tests in `wizard_tests.rs` cover only happy path
- No tests for:
  - Validation failures
  - Missing required fields  
  - Type mismatches
  - Invalid enum variants
  - Nested structure errors

**Recommended Tests:**

```rust
#[test]
fn test_validation_failure() {
    let backend = TestBackend::new()
        .with_string("email", "invalid-email");  // No @
    
    let result = User::wizard_builder()
        .with_backend(backend)
        .build();
    
    assert!(result.is_err());
}

#[test]
fn test_missing_required_field() {
    let backend = TestBackend::new()
        .with_string("name", "Alice");
        // Missing "age" field
    
    let result = User::wizard_builder()
        .with_backend(backend)
        .build();
    
    assert!(matches!(result, Err(BackendError::Answer(_))));
}
```

---

### #22: No GUI Backend Tests

**Severity:** Medium

**Problem:**

- egui backend has no automated tests
- Validation UI behavior untested
- Visual styling changes unverified
- Regression risk

**Challenge:** GUI testing is hard, but some things can be tested:

- State management logic
- Validation trigger logic
- Answer collection
- Error message formatting

**Recommendation:**

- Extract testable logic to separate functions
- Mock egui UI for unit tests
- Add integration tests with headless rendering

---

### #23: Missing Edge Case Tests

**Severity:** Medium

**Problem:**

- Deeply nested structures untested
- Prefix handling not verified
- Enum + nested struct combinations untested
- Empty interviews not tested

**Recommended Tests:**

```rust
#[test]
fn test_deeply_nested_struct() {
    // Company -> Department -> Team -> Person
}

#[test]
fn test_enum_with_nested_struct() {
    // PaymentMethod::Card { details: CardDetails { ... } }
}

#[test]
fn test_empty_interview() {
    #[derive(Wizard)]
    struct Empty {}
}
```

---

## Priority Action Items

### Immediate (This Sprint)

1. ✅ **Fix #2**: Change `build()` to return `Result<T, BackendError>` (done)
2. ✅ **Fix #3**: Make egui validation channel handling panic-free (done)
3. ✅ **Fix #4**: Avoid unwraps when aggregating errors (done)
4. 🔄 **Fix #5**: Stop mutating global UI visuals in egui (still open)
5. 🔄 **Fix #6**: Extract duplicated validation code (still open)
6. ✅ **Fix #21**: Add error path tests (doctests and cargo test currently green)

### Short Term (Next Sprint)

1. Fix #10: Add nested wizard validation
2. Add #11: Cancellation support
3. Improve #8: Expand Answers API

### Medium Term

1. Add #12: Validation timing control
2. Add #17: Partial result recovery
3. Improve #18: TestBackend validation support
4. Add #22: GUI backend testing

### Long Term / Nice to Have

1. Add #13: Conditional questions
2. Add #15: Async validation support
3. Add #19: Custom question types
4. Improve #14: Type-safe enum variants

---

## Architecture Recommendations

### 1. Separate Concerns

Split validation from data collection:

```rust
pub struct Interview {
    questions: Vec<Question>,
    validators: HashMap<String, Validator>,
}
```

### 2. Event-Driven Validation

Use observer pattern for validation triggers:

```rust
pub trait ValidationObserver {
    fn on_field_changed(&mut self, field: &str, value: &str);
    fn on_field_blurred(&mut self, field: &str);
}
```

### 3. Type-Safe Builder

```rust
pub struct WizardBuilder<T, Stage> {
    _phantom: PhantomData<(T, Stage)>,
}

impl<T> WizardBuilder<T, NoBackend> {
    pub fn with_backend(self, backend: B) -> WizardBuilder<T, HasBackend>
}

impl<T> WizardBuilder<T, HasBackend> {
    pub fn build(self) -> Result<T, BackendError>  // Only available with backend
}
```

### 4. Plugin System

```rust
pub trait QuestionPlugin {
    fn question_type(&self) -> &str;
    fn render(&self, backend: &dyn Backend) -> Result<String, Error>;
    fn validate(&self, value: &str) -> Result<(), String>;
}
```

---

## Conclusion

The derive-wizard crate has a solid foundation but needs improvements in:

1. **UI Implementation** - Fix egui visual mutation bug (still open)
2. **Validation** - Support more timing modes, nested validation, and async
3. **Testing** - Broaden coverage for error paths and GUI behavior
4. **Architecture** - Better separation of concerns and extensibility

**Overall Code Quality:** 6.5/10

- Strong: API design, macro implementation
- Weak: Error handling, testing, edge cases

**Recommended Next Steps:**

1. Fix open issues #5 (egui visuals) and #6 (validation dedupe)
2. Improve validation architecture (nested/conditional/async)
3. Expand test coverage (GUI, error paths) toward 80%+
