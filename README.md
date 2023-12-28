# Terarium

[![Tests](https://github.com/elmordo/terarium/actions/workflows/tests.yml/badge.svg)](https://github.com/elmordo/terarium/actions/workflows/tests.yml)

Terarium is library for rendering groups of templates using the [Tera](https://github.com/Keats/tera) templating library.

## Installation

```shell
cargo install terarium
```

## Usage

To create `Terarium` instance, use the `TerariumBuilder`. This builder is able to configure templates and groups and 
when you add all items, call the `build()` method to retrieve the `Terarium` instance. When preparing `Template` 
instances, you can add more than one content. Each content is bound to one language key. But language key can have 
assigned more than one content.

When instance is ready, call `render_template` or `render_group` to render single template or template group defined by 
its key. Because the library have multi-language support, the language key has to be passed and optional fallback 
language key. The fallback language is used when the primary language version of a template is not found.

Render result of the single template render is `Result<String, TerariumError>` where the `String` is the rendered 
content.

Render result of the template group render is `Result<HashMap<String, String>, TerariumError>` Where the `HashMap` 
contains the data. Keys of the hashmap is group member keys and values are their rendered contents.

## Example

```rust
use tera::Context;
use terarium::{Content, Template, TemplateGroupBuilder, TerariumBuilder};

/// The Terarium can create logical template groups and render them together,
fn main() {
  let mut builder = TerariumBuilder::default();

  builder.add_template(
    "greet_subject".to_owned(),
    Template::new(vec![
      Content::new("Greetings from {{sender}}".to_owned(), vec!["en".to_owned()]),
      Content::new("Pozdrav od {{sender}}".to_owned(), vec!["cs".to_owned()]),
    ]).unwrap(),
  ).unwrap();
  builder.add_template(
    "greet_text".to_owned(),
    Template::new(vec![
      Content::new("Hello {{username}}".to_owned(), vec!["en".to_owned()]),
      Content::new("Nazdar {{username}}".to_owned(), vec!["cs".to_owned()]),
    ]).unwrap(),
  ).unwrap();
  builder.add_template(
    "greet_html".to_owned(),
    Template::new(vec![
      Content::new("<p>Hello {{username}}</p>".to_owned(), vec!["en".to_owned()]),
      Content::new("<p>Nazdar {{username}}</p>".to_owned(), vec!["cs".to_owned()]),
    ]).unwrap()
  ).unwrap();

  builder.add_group(
    "greet_email".to_string(),
    TemplateGroupBuilder::default()
            .add_member("subject".to_owned(), "greet_subject".to_owned())
            .add_member("text".to_owned(), "greet_text".to_owned())
            .add_member("html".to_owned(), "greet_html".to_owned())
            .build(),
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
```

### Output

```text
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

## Changes in 0.3

* `Template::add_content()`, `TerariumBuilder::add_template()` and `TerariumBuilder::add_group()` methods are not
  chainable anymore. (Note: It was bad design - when method failed, instance was consumed and cannot be recovered).
* `Template::new()` constructor to partially replace removed chainable api.

For full changelog see CHANGELOG.md
