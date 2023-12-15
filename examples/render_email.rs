use terarium::{Template, TerariumBuilder};

fn main() {
    let terarium = TerariumBuilder::<String>::default()
        .add_template(
            "subject".to_owned(),
            Template::<String>::default()
                .content_builder()
                .add_content("Hello {username}".to_owned(), vec!["en".to_owned()])
                .add_content("Nazdar {username}".to_owned(), vec!["cs".to_owned()])
                .build()
        )
        .build().unwrap();
}
