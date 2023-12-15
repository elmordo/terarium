# Terarium

[![Tests](https://github.com/elmordo/terarium/actions/workflows/tests.yml/badge.svg)](https://github.com/elmordo/terarium/actions/workflows/tests.yml)

Terarium is library for rendering groups of templates using the [Tera](https://github.com/Keats/tera/tree/masterTera) templating library.

## Installation

```shell
cargo install terarium
```

## Usage

```rust
use tera::Context;
use terarium::{Template, TemplateGroupBuilder, TerariumBuilder};

fn main() {
    let terarium = TerariumBuilder::<String>::default()
        .add_template(
            "greet_subject".to_owned(),
            Template::<String>::default()
                .content_builder()
                .add_content("Greetings from {{sender}}".to_owned(), vec!["en".to_owned()])
                .add_content("Pozdrav od {{sender}}".to_owned(), vec!["cs".to_owned()])
                .build()
        )
        .add_template(
            "greet_text".to_owned(),
            Template::<String>::default()
                .content_builder()
                .add_content("Hello {{username}}".to_owned(), vec!["en".to_owned()])
                .add_content("Nazdar {{username}}".to_owned(), vec!["cs".to_owned()])
                .build()
        )
        .add_template(
            "greet_html".to_owned(),
            Template::<String>::default()
                .content_builder()
                .add_content("<p>Hello {{username}}</p>".to_owned(), vec!["en".to_owned()])
                .add_content("<p>Nazdar {{username}}</p>".to_owned(), vec!["cs".to_owned()])
                .build()
        )
        .add_group(
            "greet_email".to_string(),
            TemplateGroupBuilder::default()
                .add_member("subject".to_owned(), "greet_subject".to_owned())
                .add_member("text".to_owned(), "greet_text".to_owned())
                .add_member("html".to_owned(), "greet_html".to_owned())
                .build()
        )
        .build().unwrap();

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
```

### Output

```
English
=======

text: Hello Karel Capek
subject: Greetings from Jara Cimrman
html: <p>Hello Karel Capek</p>

Czech
=====

html: <p>Nazdar Karel Capek</p>
subject: Pozdrav od Jara Cimrman
text: Nazdar Karel Capek

```

See more examples in the project's repository.

## Note

There is no typo in name of this library. Double `r` could lead to confusion with the
[Terra](https://crates.io/crates/terra) library which is absolutely out of this library scope. :-)  
