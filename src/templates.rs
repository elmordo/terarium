use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone)]
pub struct Template<LanguageKey> {
    /// List of available contents for the template in different languages and dialects
    contents: Vec<String>,

    /// lookup of handles to real content index
    handle_to_index: HashMap<usize, usize>,

    /// Links of languages to content handles
    language_to_handle: HashMap<LanguageKey, usize>,

    /// Next handle id
    next_handle: usize,
}


/// Represent one template. The template has one or more contents. These contents are usually the same but in different
/// languages. Each content can be assigned only to one language but one language can has more then one contents.
impl<LanguageKey> Template<LanguageKey> where LanguageKey: Eq + Hash + Clone {
    /// Consume self and return the `ContentBuilder`
    pub fn content_builder(self) -> ContentBuilder<LanguageKey> {
        ContentBuilder::new(self)
    }

    /// Add new content into template.
    /// Return handle of the content.
    pub fn add_content(&mut self, content: String, languages: Vec<LanguageKey>) -> usize {
        let handle = self.next_handle;
        self.next_handle += 1;

        self.handle_to_index.insert(handle, handle);
        self.contents.push(content);
        for l in languages {
            self.language_to_handle.insert(l, handle);
        }
        handle
    }

    /// Remove content, identified by its handle, from container.
    /// If content is removed, return it.
    /// Return [`None`] if there is no content with `handle`
    pub fn remove_content(&mut self, handle: usize) -> Option<String> {
        match self.handle_to_index.remove(&handle) {
            Some(content_idx) => {
                // move all indexes after removed by one
                self.handle_to_index.iter_mut().for_each(|(_, idx)| if *idx > content_idx { *idx -= 1; });
                // clear language lookup
                self.language_to_handle.retain(|_, val| *val != handle);
                // remove the content to get return value
                let content = self.contents.remove(content_idx);
                Some(content)
            }
            _ => None
        }
    }

    /// Replace content identified by its `handle`.
    /// Return [`Some(String)`] if content was replaced.
    /// Return [`None`] if no content with given handle is defined.
    pub fn replace_content(&mut self, handle: usize, content: String) -> Option<String> {
        match self.handle_to_index.get(&handle) {
            Some(idx) => {
                let result = self.contents[*idx].clone();
                self.contents[*idx] = content;
                Some(result)
            }
            _ => None
        }
    }

    /// Reassign languages of content defined by the `handle`.
    /// Return [`Some(Vec<usize>)`] of old language settings if replacement was done successfully.
    /// Return [`None`] if given `handle` is invalid
    pub fn reassign_languages(&mut self, handle: usize, languages: Vec<LanguageKey>) -> Option<Vec<LanguageKey>> {
        match self.handle_to_index.get(&handle) {
            Some(_) => {
                let mut old_languages = Vec::new();
                self.language_to_handle = self.language_to_handle
                    .clone()
                    .into_iter()
                    .filter(|item| {
                        if item.1 == handle {
                            old_languages.push(item.0.clone());
                            false
                        } else {
                            true
                        }
                    })
                    .collect();
                for new_languages in languages {
                    self.language_to_handle.insert(new_languages, handle);
                }
                Some(old_languages)
            }
            None => None
        }
    }

    /// Collect template content settings as Vec
    /// When content has no language, this content is dropped
    pub fn collect_contents(self) -> Vec<(String, Vec<LanguageKey>)> {
        let mut languages_by_handle = HashMap::<usize, Vec<LanguageKey>>::new();
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


impl<LanguageKey> Default for Template<LanguageKey> {
    fn default() -> Self {
        Self {
            contents: Vec::new(),
            language_to_handle: HashMap::new(),
            handle_to_index: HashMap::new(),
            next_handle: 0,
        }
    }
}


/// Helper builder of easier content building
pub struct ContentBuilder<LanguageKey> where LanguageKey: Hash + Eq + Clone {
    template: Template<LanguageKey>,
}


impl<LanguageKey> ContentBuilder<LanguageKey> where LanguageKey: Hash + Eq + Clone {
    /// Create new instance from template
    pub fn new(template: Template<LanguageKey>) -> Self {
        Self {
            template,
        }
    }

    /// Add content and return self
    pub fn add_content(mut self, content: String, languages: Vec<LanguageKey>) -> Self {
        self.template.add_content(content, languages);
        self
    }

    /// Consume self and return template
    pub fn build(self) -> Template<LanguageKey> {
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
            let handle = template.add_content("foo bar".to_string(), vec![1, 2, 3]);
            assert_eq!(template.contents.len(), 1);
            assert_eq!(template.language_to_handle[&1], handle);
            assert_eq!(template.language_to_handle[&2], handle);
            assert_eq!(template.language_to_handle[&3], handle);
            let idx = template.handle_to_index[&handle];
            assert_eq!(template.contents[idx], "foo bar");
        }

        #[test]
        fn add_more_contents() {
            let mut template = empty_template();
            let handle_1 = template.add_content("foo bar".to_string(), vec![1, 2]);
            let handle_2 = template.add_content("bar foo".to_string(), vec![2, 3]);


            assert_eq!(template.language_to_handle[&1], handle_1);
            assert_eq!(template.language_to_handle[&2], handle_2);
            assert_eq!(template.language_to_handle[&3], handle_2);

            let idx_1 = template.handle_to_index[&handle_1];
            assert_eq!(template.contents[idx_1], "foo bar");
            let idx_2 = template.handle_to_index[&handle_2];
            assert_eq!(template.contents[idx_2], "bar foo");
        }

        #[test]
        fn remove_content() {
            let mut template = empty_template();
            let handle_1 = template.add_content("foo bar".to_string(), vec![1, 2]);
            let handle_2 = template.add_content("bar foo".to_string(), vec![2, 3]);
            let result = template.remove_content(handle_1);
            assert!(result.is_some());
            assert_eq!(result.unwrap(), "foo bar");

            assert_eq!(template.handle_to_index[&handle_2], 0);
            assert_eq!(template.contents[0], "bar foo");
            assert_eq!(template.handle_to_index.get(&handle_1), None);

            assert_eq!(template.language_to_handle.get(&1), None);
            assert_eq!(template.language_to_handle[&2], handle_2);
            assert_eq!(template.language_to_handle[&3], handle_2);
        }

        #[test]
        fn remove_not_existing_content() {
            let mut template = empty_template();
            let handle_1 = template.add_content("foo bar".to_string(), vec![1, 2]);
            let result = template.remove_content(handle_1 + 200);
            assert!(result.is_none());
        }

        #[test]
        fn replace_content() {
            let mut template = empty_template();
            let handle_1 = template.add_content("foo bar".to_string(), vec![1, 2]);
            let old_content = template.replace_content(handle_1, "bar bar".to_owned());

            assert!(old_content.is_some());
            assert_eq!(old_content.unwrap(), "foo bar");
            assert_eq!(template.contents[0], "bar bar");
        }

        #[test]
        fn replace_not_existing_content() {
            let mut template = empty_template();
            let handle_1 = template.add_content("foo bar".to_string(), vec![1, 2]);
            let old_content = template.replace_content(handle_1 + 100, "bar bar".to_owned());
            assert!(old_content.is_none());
        }

        #[test]
        fn reassign_languages() {
            let mut template = empty_template();
            let handle_1 = template.add_content("foo bar".to_string(), vec![1, 2]);
            let handle_2 = template.add_content("foo bar".to_string(), vec![3, 4]);
            let old_languages = template.reassign_languages(handle_1, vec![2, 5]);

            assert!(old_languages.is_some());
            let Some(mut languages) = old_languages else { panic!() };
            languages.sort();
            assert_eq!(languages, vec![1, 2]);

            assert_eq!(template.language_to_handle.get(&1), None);
            assert_eq!(template.language_to_handle[&2], handle_1);
            assert_eq!(template.language_to_handle[&5], handle_1);
            assert_eq!(template.language_to_handle[&3], handle_2);
            assert_eq!(template.language_to_handle[&4], handle_2);
        }

        #[test]
        fn reassign_not_existing_languages() {
            let mut template = empty_template();
            let handle_1 = template.add_content("foo bar".to_string(), vec![1, 2]);
            let _handle_2 = template.add_content("foo bar".to_string(), vec![3, 4]);
            let old_languages = template.reassign_languages(handle_1 + 100, vec![2, 5]);

            assert!(old_languages.is_none());
        }

        #[test]
        fn collect_contents() {
            let mut template = empty_template();
            template.add_content("foo bar".to_string(), vec![1, 2]);
            template.add_content("bar bar".to_string(), vec![3]);
            template.add_content("foo foo".to_string(), vec![]);

            let contents = template.collect_contents();
            assert_eq!(contents.len(), 2);
            let mut languages_by_content = contents.into_iter().collect::<HashMap<String, Vec<usize>>>();
            languages_by_content.values_mut().for_each(|languages| languages.sort());

            assert_eq!(languages_by_content["foo bar"], vec![1, 2]);
            assert_eq!(languages_by_content["bar bar"], vec![3]);
        }

        fn empty_template() -> Template<usize> {
            Template::default()
        }
    }
}
