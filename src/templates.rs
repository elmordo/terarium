use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct Template {
    /// List of available contents for the template in different languages and dialects
    contents: Vec<String>,

    /// lookup of handles to real content index
    handle_to_index: HashMap<usize, usize>,

    /// Links of languages to content handles
    language_to_handle: HashMap<String, usize>,

    /// Next handle id
    next_handle: usize,
}


/// Represent one template. The template has one or more contents. These contents are usually the same but in different
/// languages. Each content can be assigned only to one language but one language can has more then one contents.ash
impl Template {
    /// Consume self and return the `ContentBuilder`
    pub fn content_builder(self) -> ContentBuilder {
        ContentBuilder::new(self)
    }

    /// Add new content into template.
    /// Return handle of the content.
    pub fn add_content(&mut self, content: String, languages: Vec<String>) -> usize {
        let handle = self.next_handle;
        self.next_handle += 1;

        self.handle_to_index.insert(handle, handle);
        self.contents.push(content);
        for l in languages {
            self.language_to_handle.insert(l, handle);
        }
        handle
    }

    /// Collect template content settings as Vec
    /// When content has no language, this content is dropped
    pub fn collect_contents(self) -> Vec<(String, Vec<String>)> {
        let mut languages_by_handle = HashMap::<usize, Vec<String>>::new();
        self.language_to_handle.into_iter().for_each(|(key, handle)| {
            languages_by_handle.entry(handle).or_default().push(key);
        });
        self.handle_to_index
            .into_iter()
            .map(|(handle, idx)| {
                let content = self.contents[idx].clone();
                let languages = languages_by_handle.remove(&handle).unwrap_or(vec![]);
                (content, languages)
            })
            .filter(|(_, languages)| languages.len() > 0)
            .collect()
    }
}


/// Helper builder of easier content building
pub struct ContentBuilder {
    template: Template,
}


impl ContentBuilder {
    /// Create new instance from template
    pub fn new(template: Template) -> Self {
        Self {
            template,
        }
    }

    /// Add content and return self
    pub fn add_content(mut self, content: String, languages: Vec<String>) -> Self {
        self.template.add_content(content, languages);
        self
    }

    /// Consume self and return template
    pub fn build(self) -> Template {
        self.template
    }
}


#[cfg(test)]
mod tests {
    mod template {
        use std::collections::HashMap;

        use crate::Template;

        #[test]
        fn add_content() {
            let mut template = empty_template();
            let handle = template.add_content("foo bar".to_string(), vec!["1".to_owned(), "2".to_owned(), "3".to_owned()]);
            assert_eq!(template.contents.len(), 1);
            assert_eq!(template.language_to_handle["1"], handle);
            assert_eq!(template.language_to_handle["2"], handle);
            assert_eq!(template.language_to_handle["3"], handle);
            let idx = template.handle_to_index[&handle];
            assert_eq!(template.contents[idx], "foo bar");
        }

        #[test]
        fn add_more_contents() {
            let mut template = empty_template();
            let handle_1 = template.add_content("foo bar".to_string(), vec!["1".to_owned(), "2".to_owned()]);
            let handle_2 = template.add_content("bar foo".to_string(), vec!["2".to_owned(), "3".to_owned()]);


            assert_eq!(template.language_to_handle["1"], handle_1);
            assert_eq!(template.language_to_handle["2"], handle_2);
            assert_eq!(template.language_to_handle["3"], handle_2);

            let idx_1 = template.handle_to_index[&handle_1];
            assert_eq!(template.contents[idx_1], "foo bar");
            let idx_2 = template.handle_to_index[&handle_2];
            assert_eq!(template.contents[idx_2], "bar foo");
        }

        #[test]
        fn collect_contents() {
            let mut template = empty_template();
            template.add_content("foo bar".to_string(), vec!["1".to_owned(), "2".to_owned()]);
            template.add_content("bar bar".to_string(), vec!["3".to_owned()]);
            template.add_content("foo foo".to_string(), vec![]);

            let contents = template.collect_contents();
            assert_eq!(contents.len(), 2);
            let mut languages_by_content = contents.into_iter().collect::<HashMap<String, Vec<String>>>();
            languages_by_content.values_mut().for_each(|languages| languages.sort());

            assert_eq!(languages_by_content["foo bar"], vec!["1".to_owned(), "2".to_owned()]);
            assert_eq!(languages_by_content["bar bar"], vec!["3".to_owned()]);
        }

        fn empty_template() -> Template {
            Template::default()
        }
    }
}
