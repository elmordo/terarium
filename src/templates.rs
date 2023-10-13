use std::collections::HashMap;

pub struct TemplateRegistry<TemplateKey, LocaleKey> {
    templates: HashMap<TemplateKey, Template<LocaleKey>>,
}


pub struct Template<LocaleKey> {
    /// List of available contents for the template in different languages and dialects
    contents: Vec<String>,

    /// Links of locales to contents
    locale_links: HashMap<LocaleKey, usize>,
}
