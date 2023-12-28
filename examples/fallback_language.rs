use tera::Context;
use terarium::{Content, Template, TerariumBuilder};


/// When primary language is missing, the fallback language can be used.
fn main() {
    let mut builder = TerariumBuilder::default();
    builder.add_template(
        "my_template".to_owned(),
        Template::new(vec![Content::new("This is english template, because no czech template is available".to_owned(), vec!["en".to_owned()])]).unwrap()
    ).unwrap();
    let terarium = builder.build().unwrap();

    // The EN template will be rendered
    let result = terarium.render_template(&Context::new(), "my_template", "cs", Some("en")).unwrap();
    println!("{}", result);
}
