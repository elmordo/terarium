use tera::Context;
use terarium::{Content, Template, TemplateGroupBuilder, TerariumBuilder};

/// The Terarium can create logical template groups and render them together,
fn main() {

    let mut tpl_subject = Template::default();
    tpl_subject.add_content(Content::new("Greetings from {{sender}}".to_owned(), vec!["en".to_owned()])).unwrap();
    tpl_subject.add_content(Content::new("Pozdrav od {{sender}}".to_owned(), vec!["cs".to_owned()])).unwrap();

    let mut tpl_text = Template::default();
    tpl_text.add_content(Content::new("Hello {{username}}".to_owned(), vec!["en".to_owned()])).unwrap();
    tpl_text.add_content(Content::new("Nazdar {{username}}".to_owned(), vec!["cs".to_owned()])).unwrap();

    let mut tpl_html = Template::default();
    tpl_html.add_content(Content::new("<p>Hello {{username}}</p>".to_owned(), vec!["en".to_owned()])).unwrap();
    tpl_html.add_content(Content::new("<p>Nazdar {{username}}</p>".to_owned(), vec!["cs".to_owned()])).unwrap();

    let mut builder = TerariumBuilder::default();

    builder.add_template("greet_subject".to_owned(), tpl_subject).unwrap();
    builder.add_template("greet_text".to_owned(), tpl_text).unwrap();
    builder.add_template("greet_html".to_owned(), tpl_html).unwrap();
    builder.add_group(
            "greet_email".to_string(),
            TemplateGroupBuilder::default()
                .add_member("subject".to_owned(), "greet_subject".to_owned())
                .add_member("text".to_owned(), "greet_text".to_owned())
                .add_member("html".to_owned(), "greet_html".to_owned())
                .build()
        ).unwrap();
    let terarium = builder.build().unwrap();

    let mut ctx = Context::new();
    ctx.insert("sender", "Jara Cimrman");
    ctx.insert("username", "Karel Capek");

    let rendered_group_en = terarium.render_group(&ctx, "greet_email", "en", None).unwrap();
    let rendered_group_cs = terarium.render_group(&ctx, "greet_email", "cs", None).unwrap();

    println!("\nEnglish");
    println!("=======\n");
    rendered_group_en.iter().for_each(|(member_key, content)| println!("{}: {}", member_key, content));

    println!("\nCzech");
    println!("=====\n");
    rendered_group_cs.iter().for_each(|(member_key, content)| println!("{}: {}", member_key, content));
}
