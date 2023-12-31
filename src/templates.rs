use std::collections::HashSet;

use thiserror::Error;


/// Contains data for `Tera` template with language mutations.
#[derive(Clone, Default)]
pub struct Template {
    /// List of available contents for the template in different languages and dialects
    contents: Vec<Content>,

    /// Helper list of used languages
    used_languages: HashSet<String>,

    /// Helper list of used names
    used_names: HashSet<String>,
}


/// Represent one template. The template has one or more contents. These contents are usually the same but in different
/// languages. Each content can be assigned only to one language but one language can has more then one contents.ash
impl Template {

    /// Create instance from iterable of `Content` instances or return `Err` if creation process failed (e.g. name
    /// or language conflict).
    pub fn new(contents: Vec<Content>) -> Result<Self, TemplateError> {
        let mut instance = Self::default();
        for c in contents {
            instance.add_content(c)?;
        }
        Ok(instance)
    }

    /// Add new content into template.
    /// Return handle of the content.
    pub fn add_content(&mut self, content: Content) -> Result<(), TemplateError> {
        let mut languages_to_add = Vec::<String>::new();
        let mut names_to_add = Vec::<String>::new();

        for lang in content.languages.iter() {
            if self.used_languages.contains(lang) {
                return Err(TemplateError::DuplicatedContentLanguages(lang.to_owned()));
            }
            languages_to_add.push(lang.to_owned());
        }
        if let Some(name) = content.name.clone() {
            if self.used_names.contains(&name) {
                return Err(TemplateError::DuplicatedContentName(name));
            }
            names_to_add.push(name);
        }

        self.used_names.extend(names_to_add);
        self.used_languages.extend(languages_to_add);
        self.contents.push(content);
        Ok(())
    }

    /// Collect template content settings as Vec
    /// When content has no language, this content is dropped
    pub fn collect_contents(self) -> Vec<Content> {
        self.contents.into_iter().filter(|c| c.languages.len() > 0).collect()
    }
}


/// Errors returned by template operations.
#[derive(Debug, Error, PartialEq)]
pub enum TemplateError {
    /// Two contents in the template has same name.
    #[error("Name {0} is used by other template")]
    DuplicatedContentName(String),

    /// Two contents in the template has assigned same language.
    #[error("Language {0} is used by other template")]
    DuplicatedContentLanguages(String),
}


/// Represent content of template
#[derive(Clone, Default, Debug)]
pub struct Content {
    /// Template content.
    pub content: String,
    /// Assigned languages.
    pub languages: Vec<String>,
    /// Name of the content.
    /// The name can be used for referenced for example by {% include %} statement.
    pub name: Option<String>,
}


impl Content {
    /// Create new instance without name
    pub fn new(content: String, languages: Vec<String>) -> Self {
        Self {
            content,
            languages,
            ..Self::default()
        }
    }

    /// Create new instance with name set
    pub fn new_named(content: String, languages: Vec<String>, name: String) -> Self {
        Self {
            content,
            languages,
            name: Some(name),
        }
    }
}


#[cfg(test)]
mod tests {
    mod template {
        use std::collections::HashMap;

        use crate::{Content, Template, TemplateError};

        #[test]
        fn add_content() {
            let mut template = empty_template();
            template.add_content(Content::new("foo bar".to_string(), vec!["1".to_owned(), "2".to_owned(), "3".to_owned()])).unwrap();
            assert_eq!(template.contents.len(), 1);
        }

        #[test]
        fn add_more_contents() {
            let mut template = empty_template();
            template.add_content(Content::new("foo bar".to_string(), vec!["1".to_owned(), "2".to_owned()])).unwrap();
            template.add_content(Content::new("bar foo".to_string(), vec!["3".to_owned(), "4".to_owned()])).unwrap();
            assert_eq!(template.contents.len(), 2);
        }

        #[test]
        fn add_duplicated_name() {
            let mut tpl = empty_template();
            tpl.add_content(Content::new_named("foo".to_owned(), vec!["cs".to_owned()], "c1".to_owned())).unwrap();
            let result = tpl.add_content(Content::new_named("foo".to_owned(), vec!["en".to_owned()], "c1".to_owned()));

            assert!(result.is_err());
            let err = result.err().unwrap();
            assert_eq!(err, TemplateError::DuplicatedContentName("c1".to_owned()));
        }

        #[test]
        fn add_duplicated_language() {
            let mut tpl = empty_template();
            tpl.add_content(Content::new("foo".to_owned(), vec!["cs".to_owned()])).unwrap();
            let result = tpl.add_content(Content::new("foo".to_owned(), vec!["cs".to_owned()]));

            assert!(result.is_err());
            let err = result.err().unwrap();
            assert_eq!(err, TemplateError::DuplicatedContentLanguages("cs".to_owned()));
        }

        #[test]
        fn collect_contents() {
            let mut template = empty_template();
            template.add_content(Content::new("foo bar".to_string(), vec!["1".to_owned(), "2".to_owned()])).unwrap();
            template.add_content(Content::new("bar bar".to_string(), vec!["3".to_owned()])).unwrap();
            template.add_content(Content::new("foo foo".to_string(), vec![])).unwrap();

            let contents = template.collect_contents();
            assert_eq!(contents.len(), 2);
            let mut languages_by_content = HashMap::<String, Vec<String>>::new();

            contents.iter().for_each(|c| {
                c.languages.iter().for_each(|l| languages_by_content.entry(c.content.to_owned()).or_default().push(l.to_owned()));
            });

            assert_eq!(languages_by_content["foo bar"], vec!["1".to_owned(), "2".to_owned()]);
            assert_eq!(languages_by_content["bar bar"], vec!["3".to_owned()]);
        }

        fn empty_template() -> Template {
            Template::default()
        }
    }
}
