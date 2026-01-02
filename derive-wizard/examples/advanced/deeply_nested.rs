use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[allow(unused)]
struct GeoCoordinates {
    #[prompt("Latitude:")]
    latitude: f64,

    #[prompt("Longitude:")]
    longitude: f64,
}

#[derive(Debug, Wizard)]
#[allow(unused)]
struct Location {
    #[prompt("Location name:")]
    name: String,

    #[prompt]
    coordinates: GeoCoordinates,
}

#[derive(Debug, Wizard)]
#[allow(unused)]
struct Event {
    #[prompt("Event title:")]
    title: String,

    #[prompt("Description:")]
    description: String,

    #[prompt]
    location: Location,

    #[prompt("Max attendees:")]
    max_attendees: u32,
}

fn main() {
    let event = Event::wizard_builder().build();
    println!("{event:#?}");
}
