use derive_wizard::Wizard;

#[derive(Debug, Clone, Wizard)]
struct UserProfile {
    #[prompt("Enter your name:")]
    name: String,

    #[prompt("Enter your age:")]
    #[min(0)]
    #[max(150)]
    age: i32,

    #[prompt("Enter your email:")]
    email: String,

    #[prompt("Subscribe to newsletter:")]
    subscribe: bool,
}

fn main() {
    println!("=== Comprehensive Builder API Demo ===");

    // Example 1: Default usage (uses requestty backend by default)
    #[cfg(feature = "requestty-backend")]
    {
        println!("--- Example 1: Default Builder (Requestty) ---");
        let profile1 = UserProfile::wizard_builder().build();
        println!("Profile: {:#?}", profile1);
    }

    // Example 2: With dialoguer backend
    #[cfg(feature = "dialoguer-backend")]
    {
        println!("--- Example 2: Builder with Dialoguer Backend ---");
        let backend = derive_wizard::DialoguerBackend::new();
        let profile2 = UserProfile::wizard_builder().with_backend(backend).build();
        println!("Profile: {:#?}", profile2);
    }

    // Example 3: With egui backend
    #[cfg(feature = "egui-backend")]
    {
        println!("--- Example 3: Builder with Egui Backend ---");
        let backend = derive_wizard::EguiBackend::new()
            .with_title("User Profile")
            .with_window_size([450.0, 350.0]);

        let profile3 = UserProfile::wizard_builder().with_backend(backend).build();
        println!("Profile: {:#?}", profile3);
    }

    // Example 4: With suggestions (will be prompted with these as starting values)
    #[cfg(feature = "requestty-backend")]
    {
        println!("--- Example 4: Builder with Suggested Values ---");
        let suggestions = UserProfile {
            name: "John Doe".to_string(),
            age: 30,
            email: "john@example.com".to_string(),
            subscribe: true,
        };

        let profile4 = UserProfile::wizard_builder()
            .with_suggestions(suggestions)
            .build();
        println!("Profile: {:#?}", profile4);
    }

    // Example 5: Combining suggestions with custom backend
    #[cfg(all(feature = "requestty-backend", feature = "dialoguer-backend"))]
    {
        println!("--- Example 5: Builder with Suggestions AND Custom Backend ---");
        let suggestions = UserProfile {
            name: "Jane Smith".to_string(),
            age: 25,
            email: "jane@example.com".to_string(),
            subscribe: false,
        };

        let backend = derive_wizard::DialoguerBackend::new();

        let profile5 = UserProfile::wizard_builder()
            .with_suggestions(suggestions)
            .with_backend(backend)
            .build();
        println!("Profile: {:#?}", profile5);
    }

    println!("=== Demo Complete ===");
}
