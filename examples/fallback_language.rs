use tera::Context;
use terarium::{Content, Template, TerariumBuilder};


/// When primary language is missing, the fallback language can be used.
fn main() {
    let mut tpl = Template::default();
    tpl.add_content(Content::new("This is english template, because no czech template is available".to_owned(), vec!["en".to_owned()])).unwrap();

    let mut builder = TerariumBuilder::default();
    builder.add_template("my_template".to_owned(), tpl).unwrap();
    let terarium = builder.build().unwrap();

    // The EN template will be rendered
    let result = terarium.render_template(&Context::new(), "my_template", "cs", Some("en")).unwrap();
    println!("{}", result);
}
