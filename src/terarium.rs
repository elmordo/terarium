use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

use tera::{Context, Error as TeraError};
use tera::Tera;
use thiserror::Error;

use crate::Template;

/// Wrapper over the `Tera` templating engine with capability of template bulk rendering.
/// Each template can exists in more than one version (support for multi-language templates).
/// An instance of the `Terarium` is built with the `TerariumBuilder`.
#[derive(Clone, Default)]
pub struct Terarium {
    /// Internal Tera template
    tera: Tera,
    /// Template by template key lookup.
    template_map: HashMap<String, HashMap<String, String>>,
    /// Group by group key lookup.
    groups: HashMap<String, HashMap<String, String>>,
}

impl Terarium {
    /// Render single template identified by its key.
    /// The `Tera` context is accepted for rendering.
    pub fn render_template<K: ?Sized, LK: ?Sized>(
        &self,
        context: &Context,
        template_key: &K,
        language: &LK,
        fallback_language: Option<&LK>,
    ) -> Result<String, TerariumError>
        where
            String: Borrow<K>,
            String: Borrow<LK>,
            K: Hash + Eq,
            LK: Hash + Eq,
    {
        let template = self
            .template_map.get(template_key).ok_or_else(|| TerariumError::TemplateNotFound)?;
        let content_key = template
            .get(language)
            .or_else(|| {
                fallback_language.map(|k| template.get(k)).flatten()
            })
            .ok_or_else(|| TerariumError::LanguageNotFound)?;
        Ok(self.tera.render(content_key.as_str(), context)?)
    }

    /// Render template group.
    /// Result is HashMap where keys are member names and values are rendered templates.
    pub fn render_group<K: ?Sized, LK: ?Sized>(
        &self,
        context: &Context,
        group_key: &K,
        language: &LK,
        fallback_language: Option<&LK>,
    ) -> Result<HashMap<String, String>, TerariumError>
        where
            String: Borrow<K>,
            String: Borrow<LK>,
            K: Hash + Eq,
            LK: Hash + Eq,
    {
        let group = self.groups.get(group_key).ok_or_else(|| TerariumError::GroupNotFound)?;
        let mut result = HashMap::<String, String>::new();

        for (member_key, template_key) in group.iter() {
            let content = self.render_template(context, template_key, language, fallback_language)?;
            result.insert(member_key.clone(), content);
        }

        Ok(result)
    }
}


#[derive(Debug, Error)]
pub enum TerariumError {
    #[error("There is no template")]
    TemplateNotFound,
    #[error("Language not found")]
    LanguageNotFound,
    #[error("There is no group")]
    GroupNotFound,

    #[error("Error when rendering template")]
    RenderingFailed(TeraError),
}


impl From<TeraError> for TerariumError {
    fn from(value: TeraError) -> Self {
        Self::RenderingFailed(value)
    }
}


/// Build the `Terarium` instance.
#[derive(Default)]
pub struct TerariumBuilder {
    templates: HashMap<String, Template>,
    groups: HashMap<String, HashMap<String, String>>,
}


impl TerariumBuilder {
    /// Add new template to the new instance.
    /// If template exist, it will be replaced
    pub fn add_template(&mut self, key: String, template: Template) -> Result<(), TerariumBuilderError> {
        self.templates.insert(key.clone(), template);
        Ok(())
    }

    /// Add new group into new instance
    /// If group with same name exists, it is replaced.
    pub fn add_group(&mut self, key: String, group: HashMap<String, String>) -> Result<(), TerariumBuilderError> {
        // Check templates exist
        for (_, tpl_name) in group.iter() {
            if !self.templates.contains_key(tpl_name) {
                return Err(TerariumBuilderError::TemplateNotFound(tpl_name.to_owned()));
            }
        }

        // Add group to lookup
        self.groups.insert(key, group);
        Ok(())
    }

    /// Build new `Terarium` instance based on stored templates and groups.
    pub fn build(self) -> Result<Terarium, TerariumBuilderError> {
        let mut instance = Terarium::default();
        let mut tera_template_id: u32 = 1;

        // build templates
        self.templates.into_iter().try_for_each(|(template_key, template)| {
            template.collect_contents().into_iter().try_for_each(|content| {
                let template_name = content.name.unwrap_or_else(|| format!("template#{}", tera_template_id));
                tera_template_id += 1;
                instance.tera.add_raw_template(&template_name, &content.content)?;

                content.languages.into_iter().for_each(|language_key| {
                    instance
                        .template_map
                        .entry(template_key.clone())
                        .or_default()
                        .insert(language_key.clone(), template_name.clone());
                });

                Ok::<_, TerariumBuilderError>(())
            })?;
            Ok::<_, TerariumBuilderError>(())
        })?;

        instance.groups = self.groups;
        Ok(instance)
    }
}


/// Simplify building template groups.
#[derive(Clone, Default)]
pub struct TemplateGroupBuilder {
    group: HashMap<String, String>,
}

impl TemplateGroupBuilder {
    /// Add new member to group.
    pub fn add_member(mut self, member_key: String, template_key: String) -> Self {
        self.group.insert(member_key, template_key);
        self
    }

    /// Build the group spec.
    pub fn build(self) -> HashMap<String, String> {
        self.group
    }
}


#[derive(Debug, Error)]
pub enum TerariumBuilderError {
    #[error("Unable to build template")]
    TemplateBuildingError(TeraError),
    #[error("Cannot build template groups - some templates are missing")]
    TemplateNotFound(String),
}


impl From<TeraError> for TerariumBuilderError {
    fn from(value: TeraError) -> Self {
        Self::TemplateBuildingError(value)
    }
}


/// Additional methods for testing
#[cfg(test)]
impl TerariumBuilder {
    /// Get template defined by its `key`.
    /// If no template defined by given `key` exist, return `None`.
    pub fn get_template(&mut self, key: &String) -> Option<&mut Template> {
        self.templates.get_mut(key)
    }

    /// Remove template defined by the `key` from the builder and return it.
    /// Returns `None` if no template with given `key` is defined.
    pub fn remove_template(&mut self, key: &String) -> Option<Template> {
        self.templates.remove(key)
    }

    /// Get group defined by the `key`.
    /// Return `None` if no group defined by the `key` is found.
    pub fn get_group(&mut self, key: &String) -> Option<&mut HashMap<String, String>> {
        self.groups.get_mut(key)
    }

    /// Remove group defined by the `key` from the builder and return it.
    /// Returns `None` if no group with given `key` is defined.
    pub fn remove_group(&mut self, key: &String) -> Option<HashMap<String, String>> {
        self.groups.remove(key)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    mod terarium_builder {
        use crate::Content;

        use super::*;

        #[test]
        fn add_template() {
            let mut instance = make_instance();

            let mut tpl = Template::default();
            tpl.add_content(Content::new("foo".to_string(), vec!["1".to_owned(), "2".to_owned()])).unwrap();

            instance.add_template(
                "1".to_owned(),
                tpl,
            ).unwrap();

            assert_eq!(instance.templates.len(), 1);
            let template = instance.templates["1"].clone();
            let contents = template.collect_contents();
            assert_eq!(contents.len(), 1);
        }

        #[test]
        fn group_manipulation() {
            let mut instance = make_instance();
            instance.add_template("1".to_owned(), Template::default()).unwrap();
            instance.add_group("1".to_owned(), TemplateGroupBuilder::default().add_member("1".to_owned(), "1".to_owned()).build()).unwrap();
            let grp = instance.get_group(&"1".to_owned());
            assert!(grp.is_some());
            let grp = grp.unwrap();
            assert_eq!(grp.clone(), HashMap::<String, String>::from([("1".to_owned(), "1".to_owned())]));

            instance.remove_group(&"1".to_owned());
            assert!(instance.get_group(&"1".to_owned()).is_none())
        }

        #[test]
        fn check_group_configuration() {
            let mut instance = make_instance();
            instance.add_template("1".to_owned(), Template::default()).unwrap();
            instance.add_template("2".to_owned(), Template::default()).unwrap();
            let result = instance.add_group(
                "100".to_owned(),
                TemplateGroupBuilder::default()
                    .add_member("10".to_owned(), "1".to_owned())
                    .add_member("20".to_owned(), "2".to_owned())
                    .add_member("30".to_owned(), "3".to_owned())
                    .build(),
            );
            assert!(result.is_err())
        }

        fn make_instance() -> TerariumBuilder {
            TerariumBuilder::default()
        }
    }

    mod terarium {
        use crate::Content;

        use super::*;

        #[test]
        fn render_template() {
            let instance = make_instance();
            let ctx = make_context();
            let result_a = instance.render_template(&ctx, "template_a", "cs", None).unwrap();
            assert_eq!(result_a, "template_a cs john");
        }

        #[test]
        fn render_template_with_fallback() {
            let instance = make_instance();
            let ctx = make_context();
            let result_a = instance.render_template(&ctx, "template_a", "de", Some("en")).unwrap();
            assert_eq!(result_a, "template_a en john");
        }

        #[test]
        fn render_template_without_matching_language() {
            let instance = make_instance();
            let ctx = make_context();
            let result = instance.render_template(&ctx, "template_a", "de", Some("fr"));

            assert!(match result.unwrap_err() {
                TerariumError::LanguageNotFound => true,
                _ => false
            })
        }

        #[test]
        fn render_group() {
            let instance = make_instance();
            let context = make_context();
            let group_result = instance.render_group(&context, "group_a", "en", None);
            assert!(group_result.is_ok());
            let group_result = group_result.unwrap();
            assert_eq!(group_result.get("A").unwrap(), "template_a en john");
            assert_eq!(group_result.get("B").unwrap(), "template_b en doe");
        }

        #[test]
        fn render_group_with_fallback() {
            let instance = make_instance();
            let context = make_context();
            let group_result = instance.render_group(&context, "group_a", "cs", Some("en"));
            assert!(group_result.is_ok());
            let group_result = group_result.unwrap();
            assert_eq!(group_result.get("A").unwrap(), "template_a cs john");
            assert_eq!(group_result.get("B").unwrap(), "template_b en doe");
        }

        #[test]
        fn render_group_when_invalid_language() {
            let instance = make_instance();
            let context = make_context();
            let group_result = instance.render_group(&context, "group_a", "cs", Some("fr"));
            assert!(group_result.is_err());
            assert!(match group_result.unwrap_err() {
                TerariumError::LanguageNotFound => true,
                _ => false
            })
        }

        #[test]
        fn render_nested_templates() {
            let mut builder = TerariumBuilder::default();
            let mut tpl_a = Template::default();
            tpl_a.add_content(
                Content::new("This is content {{value_1}} {% include 'tpl_b_cs' %}".to_owned(), vec!["cs".to_owned()])
            ).unwrap();
            builder.add_template("tpl_a".to_owned(), tpl_a).unwrap();

            let mut tpl_b = Template::default();
            tpl_b.add_content(
                Content::new_named("This is nested {{value_2}}".to_owned(), vec!["cs".to_owned()], "tpl_b_cs".to_owned())
            ).unwrap();
            builder.add_template("tpl_b".to_owned(), tpl_b).unwrap();

            let instance = builder.build().unwrap();
            let mut ctx = Context::default();
            ctx.insert("value_1", "foo");
            ctx.insert("value_2", "bar");

            let result = instance.render_template(&ctx, "tpl_a", "cs", None).unwrap();
            assert_eq!(result.as_str(), "This is content foo This is nested bar");
        }

        fn make_instance() -> Terarium {
            let mut builder = TerariumBuilder::default();

            let mut tpl_a = Template::default();
            tpl_a.add_content(Content::new("template_a cs {{name}}".to_owned(), vec!["cs".to_owned()])).unwrap();
            tpl_a.add_content(Content::new("template_a en {{name}}".to_owned(), vec!["en".to_owned()])).unwrap();

            let mut tpl_b = Template::default();
            tpl_b.add_content(Content::new("template_b en {{surname}}".to_owned(), vec!["en".to_owned()])).unwrap();

            builder.add_template("template_a".to_owned(), tpl_a).unwrap();
            builder.add_template("template_b".to_owned(), tpl_b).unwrap();

            builder.add_group(
                "group_a".to_owned(),
                TemplateGroupBuilder::default()
                    .add_member("A".to_owned(), "template_a".to_owned())
                    .add_member("B".to_owned(), "template_b".to_owned())
                    .build(),
            ).unwrap();
            builder.build().unwrap()
        }

        fn make_context() -> Context {
            let mut ctx = Context::default();
            ctx.insert("name", "john");
            ctx.insert("surname", "doe");
            ctx
        }
    }
}
