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
                self.locale_to_handle.retain(|key, val| *val == handle);
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

    /// Get content for locale or for fallback.
    /// Return the content if `locale` or `fallback_locale` is found.
    /// Return [`None`] if there is no suitable content.
    pub fn get_content(&self, locale: LocaleKey, fallback_locale: Option<LocaleKey>) -> Option<String> {
        self.locale_to_handle
            .get(&locale)
            .or_else(|| {
                match fallback_locale {
                    Some(key) => self.locale_to_handle.get(&key),
                    None => None
                }
            }).map(|idx| self.contents[*idx].clone())
    }
}
