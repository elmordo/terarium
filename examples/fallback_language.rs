use tera::Context;
use terarium::{Template, TerariumBuilder};

fn main() {
    let terarium = TerariumBuilder::default()
        .add_template(
            "my_template".to_owned(),
            Template::default().content_builder()
                .add_content("This is english template, because no czech template is available".to_owned(), vec!["en".to_owned()])
                .build()
        )
        .build().unwrap();

    // The EN template will be rendered
    let result = terarium.render_template(&Context::new(), "my_template", "cs", Some("en")).unwrap();
    println!("{}", result);
}
