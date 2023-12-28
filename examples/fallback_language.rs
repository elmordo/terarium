use tera::Context;
use terarium::{Content, Template, TerariumBuilder};


/// When primary language is missing, the fallback language can be used.
fn main() {
    let terarium = TerariumBuilder::default()
        .add_template(
            "my_template".to_owned(),
            Template::default()
                .add_content(Content::new("This is english template, because no czech template is available".to_owned(), vec!["en".to_owned()])).unwrap()
        )
        .build().unwrap();

    // The EN template will be rendered
    let result = terarium.render_template(&Context::new(), "my_template", "cs", Some("en")).unwrap();
    println!("{}", result);
}
