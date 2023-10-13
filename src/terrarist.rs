use std::collections::HashMap;

use crate::templates::TemplateRegistry;

pub struct Terrarist<GroupKey, TemplateKey, LocaleKey> {
    fallback_locale: Option<LocaleKey>,
    templates: TemplateRegistry<TemplateKey, LocaleKey>,
    groups: HashMap<GroupKey, Vec<TemplateKey>>,
}
