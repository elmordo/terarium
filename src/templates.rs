use std::collections::HashMap;
use std::hash::Hash;

pub struct TemplateRegistry<TemplateKey, LocaleKey> {
    templates: HashMap<TemplateKey, Template<LocaleKey>>,
}


#[derive(Clone, Default)]
pub struct Template<LocaleKey> {
    /// List of available contents for the template in different languages and dialects
    contents: Vec<String>,

    /// lookup of handles to real content index
    handle_to_index: HashMap<usize, usize>,

    /// Links of locales to content handles
    locale_to_handle: HashMap<LocaleKey, usize>,

    /// Next handle id
    next_handle: usize,
}


impl<LocaleKey> Template<LocaleKey> where LocaleKey: Eq + Hash + Copy {
    /// Add new content into template.
    /// Return handle of the content.
    pub fn add_content(&mut self, content: String, locales: Vec<LocaleKey>) -> usize {
        let handle = self.next_handle;
        self.next_handle += 1;

        self.handle_to_index.insert(handle, handle);
        self.contents.push(content);
        for l in locales {
            self.locale_to_handle.insert(l, handle);
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
                // clear locale lookup
                self.locale_to_handle.retain(|_, val| *val != handle);
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

    /// Reassign locales of content defined by the `handle`.
    /// Return [`Some(Vec<usize>)`] of old locales settings if replacement was done successfully.
    /// Return [`None`] if given `handle` is invalid
    pub fn reassign_locales(&mut self, handle: usize, locales: Vec<LocaleKey>) -> Option<Vec<LocaleKey>> {
        match self.handle_to_index.get(&handle) {
            Some(_) => {
                let mut old_locales = Vec::new();
                self.locale_to_handle = self.locale_to_handle
                    .clone()
                    .into_iter()
                    .filter(|item| {
                        if item.1 == handle {
                            old_locales.push(item.0);
                            false
                        } else {
                            true
                        }
                    })
                    .collect();
                for new_locale in locales {
                    self.locale_to_handle.insert(new_locale, handle);
                }
                Some(old_locales)
            }
            None => None
        }
    }

    /// Collect template content settings as Vec
    /// When content has no locale, this content is dropped
    pub fn collect_contents(self) -> Vec<(String, Vec<LocaleKey>)> {
        let mut locales_by_handle = HashMap::<usize, Vec<LocaleKey>>::new();
        self.locale_to_handle.into_iter().for_each(|(key, handle)| {
            locales_by_handle.entry(handle).or_default().push(key);
        });
        self.handle_to_index
            .into_iter()
            .map(|(handle, idx)| {
                let content = self.contents[idx].clone();
                let locales = locales_by_handle.remove(&handle).unwrap_or(vec![]);
                (content, locales)
            })
            .filter(|(_, locales)| locales.len() > 0)
            .collect()
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
            assert_eq!(template.locale_to_handle[&1], handle);
            assert_eq!(template.locale_to_handle[&2], handle);
            assert_eq!(template.locale_to_handle[&3], handle);
            let idx = template.handle_to_index[&handle];
            assert_eq!(template.contents[idx], "foo bar");
        }

        #[test]
        fn add_more_contents() {
            let mut template = empty_template();
            let handle_1 = template.add_content("foo bar".to_string(), vec![1, 2]);
            let handle_2 = template.add_content("bar foo".to_string(), vec![2, 3]);


            assert_eq!(template.locale_to_handle[&1], handle_1);
            assert_eq!(template.locale_to_handle[&2], handle_2);
            assert_eq!(template.locale_to_handle[&3], handle_2);

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

            assert_eq!(template.locale_to_handle.get(&1), None);
            assert_eq!(template.locale_to_handle[&2], handle_2);
            assert_eq!(template.locale_to_handle[&3], handle_2);
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
        fn reassign_locales() {
            let mut template = empty_template();
            let handle_1 = template.add_content("foo bar".to_string(), vec![1, 2]);
            let handle_2 = template.add_content("foo bar".to_string(), vec![3, 4]);
            let old_locales = template.reassign_locales(handle_1, vec![2, 5]);

            assert!(old_locales.is_some());
            let Some(mut locales) = old_locales else { panic!() };
            locales.sort();
            assert_eq!(locales, vec![1, 2]);

            assert_eq!(template.locale_to_handle.get(&1), None);
            assert_eq!(template.locale_to_handle[&2], handle_1);
            assert_eq!(template.locale_to_handle[&5], handle_1);
            assert_eq!(template.locale_to_handle[&3], handle_2);
            assert_eq!(template.locale_to_handle[&4], handle_2);
        }

        #[test]
        fn reassign_not_existing_locales() {
            let mut template = empty_template();
            let handle_1 = template.add_content("foo bar".to_string(), vec![1, 2]);
            let handle_2 = template.add_content("foo bar".to_string(), vec![3, 4]);
            let old_locales = template.reassign_locales(handle_1 + 100, vec![2, 5]);

            assert!(old_locales.is_none());
        }

        #[test]
        fn collect_contents() {
            let mut template = empty_template();
            template.add_content("foo bar".to_string(), vec![1, 2]);
            template.add_content("bar bar".to_string(), vec![3]);
            template.add_content("foo foo".to_string(), vec![]);

            let contents = template.collect_contents();
            assert_eq!(contents.len(), 2);
            let mut locales_by_content = contents.into_iter().collect::<HashMap<String, Vec<usize>>>();
            locales_by_content.values_mut().for_each(|locales| locales.sort());

            assert_eq!(locales_by_content["foo bar"], vec![1, 2]);
            assert_eq!(locales_by_content["bar bar"], vec![3]);
        }

        fn empty_template() -> Template<usize> {
            Template::default()
        }
    }
}
