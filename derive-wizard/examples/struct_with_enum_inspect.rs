use derive_wizard::Wizard;

#[derive(Debug, Clone, Wizard)]
#[allow(unused)]
enum PaymentMethod {
    Cash,

    CreditCard {
        #[prompt("Credit Card Number:")]
        card_number: String,

        #[prompt("Expiry (MM/YY):")]
        expiry: String,

        #[prompt("CVV:")]
        #[mask]
        cvv: String,
    },

    BankTransfer {
        #[prompt("Bank Name:")]
        bank_name: String,

        #[prompt("Account Number:")]
        account_number: String,

        #[prompt("Routing Number:")]
        routing_number: String,
    },
}

#[derive(Debug, Clone, Wizard)]
#[allow(unused)]
struct Order {
    #[prompt("Product name:")]
    product: String,

    #[prompt("Quantity:")]
    #[min(1)]
    #[max(100)]
    quantity: i32,

    #[prompt("Price per item:")]
    #[min(0.01)]
    price: f64,

    #[prompt("Enable gift wrapping:")]
    gift_wrap: bool,
}

fn print_question(question: &derive_wizard::interview::Question, indent: usize) {
    let prefix = "  ".repeat(indent);
    println!("{}Question: {}", prefix, question.name());
    println!("{}  ID: {:?}", prefix, question.id());
    println!("{}  Prompt: '{}'", prefix, question.prompt());

    use derive_wizard::interview::QuestionKind;
    match question.kind() {
        QuestionKind::Input(q) => {
            println!("{}  Type: Input", prefix);
            if let Some(default) = &q.default {
                println!("{}    Default: '{}'", prefix, default);
            }
        }
        QuestionKind::Multiline(q) => {
            println!("{}  Type: Multiline", prefix);
            if let Some(default) = &q.default {
                println!("{}    Default: '{}'", prefix, default);
            }
        }
        QuestionKind::Masked(q) => {
            println!("{}  Type: Masked (password)", prefix);
            if let Some(mask) = q.mask {
                println!("{}    Mask character: '{}'", prefix, mask);
            }
        }
        QuestionKind::Int(q) => {
            println!("{}  Type: Integer", prefix);
            if let Some(default) = q.default {
                println!("{}    Default: {}", prefix, default);
            }
            if let Some(min) = q.min {
                println!("{}    Min: {}", prefix, min);
            }
            if let Some(max) = q.max {
                println!("{}    Max: {}", prefix, max);
            }
        }
        QuestionKind::Float(q) => {
            println!("{}  Type: Float", prefix);
            if let Some(default) = q.default {
                println!("{}    Default: {}", prefix, default);
            }
            if let Some(min) = q.min {
                println!("{}    Min: {}", prefix, min);
            }
            if let Some(max) = q.max {
                println!("{}    Max: {}", prefix, max);
            }
        }
        QuestionKind::Confirm(q) => {
            println!("{}  Type: Confirm (yes/no)", prefix);
            println!("{}    Default: {}", prefix, q.default);
        }
        QuestionKind::Sequence(questions) => {
            println!("{}  Type: Sequence ({} questions)", prefix, questions.len());
            for (idx, q) in questions.iter().enumerate() {
                println!("{}  [Sequence item {}]", prefix, idx);
                print_question(q, indent + 2);
            }
        }
        QuestionKind::Alternative(default_idx, alternatives) => {
            println!(
                "{}  Type: Alternative (enum with {} variants, default: {})",
                prefix,
                alternatives.len(),
                default_idx
            );
            for (idx, alt) in alternatives.iter().enumerate() {
                println!("{}  [Variant {}] {}", prefix, idx, alt.name());
                print_question(alt, indent + 2);
            }
        }
    }
    println!();
}

fn main() {
    println!("=== Struct with Enum - Interview Inspection ===\n");

    println!("--- Order Interview Structure ---\n");
    let order_interview = Order::interview();
    println!(
        "Number of questions in Order: {}\n",
        order_interview.sections.len()
);

    for (idx, question) in order_interview.sections.iter().enumerate() {
        println!("[Question {}]", idx);
        print_question(question, 0);
    }

    println!("\n--- PaymentMethod Interview Structure ---\n");
    let payment_interview = PaymentMethod::interview();
    println!(
        "Number of questions in PaymentMethod: {}\n",
        payment_interview.sections.len()
);

    for (idx, question) in payment_interview.sections.iter().enumerate() {
        println!("[Question {}]", idx);
        print_question(question, 0);
    }

    println!("=== End of Inspection ===");
}
