use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[allow(unused)]
struct Article {
    #[prompt("Enter the article title:")]
    title: String,

    #[prompt("Write the article content:")]
    #[editor]
    content: String,

    #[prompt("Add tags (comma-separated):")]
    tags: String,
}

fn main() {
    println!("Article Wizard Demo");
    println!("This demonstrates the #[editor] attribute");
    println!("which opens your preferred text editor for longer input.");

    let article = Article::wizard_builder().build();
    println!("{article:#?}");
}
