use tera::Context;
use terarium::{Template, TerariumBuilder};

fn main() {
    let terarium = TerariumBuilder::default()
        .add_template(
            "my_template".to_owned(),
            Template::default().content_builder()
                .add_content("This is my template #{{tpl_number}}".to_owned(), vec!["en".to_owned()])
                .add_content("Toto je šablona #{{tpl_number}}".to_owned(), vec!["cs".to_owned()])
                .build()
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
