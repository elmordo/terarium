use tera::Context;

use terarium::{Content, Template, TerariumBuilder};

/// The Terarium can render single template.
fn main() {
    let mut builder = TerariumBuilder::default();
    builder.add_template(
        "my_template".to_owned(),
        Template::new(vec![
            Content::new("This is my template #{{tpl_number}}".to_owned(), vec!["en".to_owned()]),
            Content::new("Toto je šablona #{{tpl_number}}".to_owned(), vec!["cs".to_owned()]),
        ]).unwrap()
    ).unwrap();

    let terarium = builder.build().unwrap();

    let mut ctx = Context::new();
    ctx.insert("tpl_number", "13");

    let output_en = terarium.render_template(&ctx, "my_template", "en", None).unwrap();
    let output_cs = terarium.render_template(&ctx, "my_template", "cs", None).unwrap();

    println!("\nEnglish:\n");
    println!("{}\n", output_en);

    println!("\nCzech:\n");
    println!("{}\n", output_cs);
}
