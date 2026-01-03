//! Simplified rental application form example

use derive_wizard::Wizard;

#[derive(Wizard, Debug)]
#[prelude("Please complete all sections of this application.")]
#[epilogue("Thank you for applying!")]
#[allow(unused)]
struct RentalApplication {
    #[prompt("Full Legal Name")]
    full_name: String,

    #[prompt("Email Address")]
    email: String,

    #[prompt("Phone Number")]
    phone: String,

    #[prompt("Employment Status")]
    employment: EmploymentStatus,
}

#[derive(Wizard, Debug)]
#[allow(unused)]
enum EmploymentStatus {
    #[prompt("Employed")]
    Employed {
        #[prompt("Employer Name")]
        employer: String,

        #[prompt("Monthly Income")]
        #[min(0)]
        income: i64,
    },

    #[prompt("Self-Employed")]
    SelfEmployed {
        #[prompt("Business Name")]
        business: String,
    },

    #[prompt("Unemployed")]
    Unemployed,
}

fn main() {
    #[cfg(feature = "typst-form")]
    {
        let typst_markup = RentalApplication::to_typst_form(Some("RENTAL APPLICATION"));
        std::fs::write("rental_application.typ", &typst_markup)
            .expect("Failed to write rental_application.typ");
        println!("✓ Generated Typst form: rental_application.typ");
    }
}
