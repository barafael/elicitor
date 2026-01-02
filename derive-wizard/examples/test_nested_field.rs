use derive_wizard::{FieldPath, Wizard, field};

#[derive(Debug, PartialEq, Wizard)]
struct Address {
    #[prompt("Street:")]
    street: String,
    #[prompt("City:")]
    city: String,
}

#[derive(Debug, PartialEq, Wizard)]
struct UserWithAddress {
    #[prompt("Name:")]
    name: String,
    address: Address,
}

fn main() {
    // Test the field macro
    let path: FieldPath = field!(UserWithAddress::address::street);
    println!("Field path: {:?}", path);
    println!("Segments: {:?}", path.segments());
    println!("Slash path: {}", path.to_slash_path());
    println!("Depth: {}", path.depth());

    println!("--- Interview Structure ---");
    let interview = UserWithAddress::interview();
    println!("Number of top-level sections: {}", interview.sections.len());

    for (i, q) in interview.sections.iter().enumerate() {
        println!(
            "Section {}: name='{}', prompt='{}'",
            i,
            q.name(),
            q.prompt()
        );
        print_question_kind(q.kind(), 1);
    }
}

fn print_question_kind(kind: &derive_wizard::interview::QuestionKind, indent: usize) {
    use derive_wizard::interview::QuestionKind;

    let prefix = "  ".repeat(indent);
    match kind {
        QuestionKind::Input(_) => println!("{}Kind: Input", prefix),
        QuestionKind::Multiline(_) => println!("{}Kind: Multiline", prefix),
        QuestionKind::Masked(_) => println!("{}Kind: Masked", prefix),
        QuestionKind::Int(_) => println!("{}Kind: Int", prefix),
        QuestionKind::Float(_) => println!("{}Kind: Float", prefix),
        QuestionKind::Confirm(_) => println!("{}Kind: Confirm", prefix),
        QuestionKind::Sequence(questions) => {
            println!(
                "{}Kind: Sequence ({} nested questions)",
                prefix,
                questions.len()
            );
            for (j, nested) in questions.iter().enumerate() {
                println!(
                    "{}  [{}] name='{}', prompt='{}'",
                    prefix,
                    j,
                    nested.name(),
                    nested.prompt()
                );
                print_question_kind(nested.kind(), indent + 2);
            }
        }
        QuestionKind::Alternative(idx, alts) => {
            println!(
                "{}Kind: Alternative (default={}, {} alternatives)",
                prefix,
                idx,
                alts.len()
            );
        }
    }
}
