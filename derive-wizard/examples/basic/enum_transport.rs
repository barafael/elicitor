use derive_wizard::Wizard;

/// Bike type - Road or Mountain
#[derive(Debug, Wizard)]
enum BikeType {
    Road,
    Mountain,
}

/// Different modes of transportation
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
        #[prompt("Bike type:")]
        typ: BikeType,

        #[prompt("Number of gears:")]
        gears: u8,
    },

    Walk,
}

fn main() {
    let transport = Transport::wizard_builder().build().unwrap();
    println!("Transport: {:#?}", transport);
}
