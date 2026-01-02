use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[allow(unused)]
enum Transport {
    Car {
        #[prompt("Model:")]
        model: String,

        #[prompt("Year:")]
        year: u32,
    },

    Bicycle {
        #[prompt("Type (road/mountain):")]
        bike_type: String,

        #[prompt("Gears:")]
        gears: u8,
    },

    Walk,
}

fn main() {
    let transport = Transport::wizard_builder().build();
    println!("{:#?}", transport);
}
