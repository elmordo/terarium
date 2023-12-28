use tera::Context;
use terarium::{Content, Template, TerariumBuilder};

/// The Terarium can render single template.
fn main() {
    let terarium = TerariumBuilder::default()
        .add_template(
            "my_template".to_owned(),
            Template::default()
                .add_content(Content::new("This is my template #{{tpl_number}}".to_owned(), vec!["en".to_owned()])).unwrap()
                .add_content(Content::new("Toto je šablona #{{tpl_number}}".to_owned(), vec!["cs".to_owned()])).unwrap()
        )
        .build().unwrap();

    let mut ctx = Context::new();
    ctx.insert("tpl_number", "13");

    let output_en = terarium.render_template(&ctx, "my_template", "en", None).unwrap();
    let output_cs = terarium.render_template(&ctx, "my_template", "cs", None).unwrap();

    println!("\nEnglish:\n");
    println!("{}\n", output_en);

    println!("\nCzech:\n");
    println!("{}\n", output_cs);
}
