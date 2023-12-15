use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

use tera::{Context, Error as TeraError};
use tera::Tera;
use thiserror::Error;

use crate::Template;

pub struct Terarium<KeyType>
    where
        KeyType: Eq + Hash + Clone,
{
    tera: Tera,
    template_map: HashMap<KeyType, HashMap<KeyType, String>>,
    groups: HashMap<KeyType, HashMap<KeyType, KeyType>>,
}

impl<KeyType> Terarium<KeyType>
    where
        KeyType: Eq + Hash + Clone,
{
    pub fn render_template<K: ?Sized, LK: ?Sized>(
        &self,
        context: &Context,
        template_key: &K,
        language: &LK,
        fallback_language: Option<&LK>,
    ) -> Result<String, TerariumError>
        where
            KeyType: Borrow<K>,
            KeyType: Borrow<LK>,
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

    pub fn render_group<K: ?Sized, LK: ?Sized>(
        &self,
        context: &Context,
        group_key: &K,
        language: &LK,
        fallback_language: Option<&LK>,
    ) -> Result<HashMap<KeyType, String>, TerariumError>
        where
            KeyType: Borrow<K>,
            KeyType: Borrow<LK>,
            K: Hash + Eq,
            LK: Hash + Eq,
    {
        let group = self.groups.get(group_key).ok_or_else(|| TerariumError::GroupNotFound)?;
        let mut result = HashMap::<KeyType, String>::new();

        for (member_key, template_key) in group.iter() {
            let content = self.render_template(context, template_key, language, fallback_language)?;
            result.insert(member_key.clone(), content);
        }

        Ok(result)
    }
}


impl<KeyType> Default for Terarium<KeyType>
    where
        KeyType: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self {
            tera: Tera::default(),
            template_map: HashMap::new(),
            groups: HashMap::new(),
        }
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


#[derive(Default)]
pub struct TerariumBuilder<KeyType>
    where
        KeyType: Eq + Hash + Clone,
{
    templates: HashMap<KeyType, Template<KeyType>>,
    groups: HashMap<KeyType, HashMap<KeyType, KeyType>>,
}


impl<KeyType> TerariumBuilder<KeyType>
    where
        KeyType: Eq + Hash + Clone,
{
    pub fn add_template(mut self, key: KeyType, template: Template<KeyType>) -> Self {
        self.templates.insert(key.clone(), template);
        self
    }

    pub fn add_group(mut self, key: KeyType, group: HashMap<KeyType, KeyType>) -> Self {
        self.groups.insert(key.clone(), group);
        self
    }

    pub fn check_group_config_validity(&self) -> Vec<(KeyType, KeyType, KeyType)> {
        self.groups
            .iter()
            .map(|(group_key, members)| {
                // Check the group `group_key` and iterate over members
                // missing templates are returned as iterable. This iterable is used as the
                // `map` output
                members
                    .iter()
                    .filter(|(_, template)| !self.templates.contains_key(*template))
                    .map(|(member, template)| (group_key.clone(), member.clone(), template.clone()))
            })
            .flatten()  // Concat iterable of iterables into final output form
            .collect()
    }

    pub fn build(self) -> Result<Terarium<KeyType>, TerariumBuilderError<KeyType>> {
        let check_result = self.check_group_config_validity();
        if !check_result.is_empty() {
            return Err(TerariumBuilderError::GroupIntegrityProblem(check_result));
        }

        let mut instance = Terarium::default();
        let mut tera_template_id: u32 = 1;

        // build templates
        self.templates.into_iter().try_for_each(|(template_key, template)| {
            template.collect_contents().into_iter().try_for_each(|(content, languages)| {
                let template_name = format!("template#{}", tera_template_id);
                tera_template_id += 1;
                instance.tera.add_raw_template(&template_name, &content)?;

                languages.into_iter().for_each(|language_key| {
                    instance
                        .template_map
                        .entry(template_key.clone())
                        .or_default()
                        .insert(language_key.clone(), template_name.clone());
                });

                Ok::<_, TerariumBuilderError<_>>(())
            })?;
            Ok::<_, TerariumBuilderError<_>>(())
        })?;

        instance.groups = self.groups;
        Ok(instance)
    }
}


pub struct TemplateGroupBuilder<KeyType> where KeyType: Hash + Eq + Clone {
    group: HashMap<KeyType, KeyType>,
}

impl<KeyType> TemplateGroupBuilder<KeyType> where KeyType: Hash + Eq + Clone {
    pub fn add_member(mut self, name: KeyType, template_key: KeyType) -> Self {
        self.group.insert(name, template_key);
        self
    }

    pub fn build(self) -> HashMap<KeyType, KeyType> {
        self.group
    }
}


impl<KeyType> Default for TemplateGroupBuilder<KeyType> where KeyType: Hash + Eq + Clone {
    fn default() -> Self {
        Self {
            group: HashMap::new(),
        }
    }
}


#[derive(Debug, Error)]
pub enum TerariumBuilderError<KeyType> {
    #[error("Unable to build template")]
    TemplateBuildingError(TeraError),
    #[error("Cannot build template groups - some templates are missing")]
    GroupIntegrityProblem(Vec<(KeyType, KeyType, KeyType)>),
}


impl<KeyType> From<TeraError> for TerariumBuilderError<KeyType> {
    fn from(value: TeraError) -> Self {
        Self::TemplateBuildingError(value)
    }
}


/// Additional methods for testing
#[cfg(test)]
impl<KeyType> TerariumBuilder<KeyType>
    where
        KeyType: Eq + Hash + Clone, {

    pub fn get_template(&mut self, key: &KeyType) -> Option<&mut Template<KeyType>> {
        self.templates.get_mut(key)
    }

    pub fn remove_template(&mut self, key: &KeyType) -> Option<Template<KeyType>> {
        self.templates.remove(key)
    }

    pub fn get_group(&mut self, key: &KeyType) -> Option<&mut HashMap<KeyType, KeyType>> {
        self.groups.get_mut(key)
    }

    pub fn remove_group(&mut self, key: &KeyType) -> Option<HashMap<KeyType, KeyType>> {
        self.groups.remove(key)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    mod terarium_builder {
        use super::*;

        #[test]
        fn add_template() {
            let mut instance = make_instance();
            instance = instance.add_template(
                1,
                Template::default()
                    .content_builder()
                    .add_content("foo".to_string(), vec![1, 2])
                    .build(),
            );

            assert_eq!(instance.templates.len(), 1);
            let template = instance.templates[&1].clone();
            let mut contents = template.collect_contents();
            assert_eq!(contents.len(), 1);
            contents[0].1.sort();
            assert_eq!(contents, vec![("foo".to_string(), vec![1, 2])]);
        }

        #[test]
        fn group_manipulation() {
            let mut instance = make_instance();
            instance = instance.add_group(1, TemplateGroupBuilder::default().add_member(1, 1).build());
            let grp = instance.get_group(&1);
            assert!(grp.is_some());
            let grp = grp.unwrap();
            assert_eq!(grp.clone(), HashMap::<usize, usize>::from([(1, 1)]));

            instance.remove_group(&1);
            assert!(instance.get_group(&1).is_none())
        }

        #[test]
        fn check_group_configuration() {
            let mut instance = make_instance();
            instance = instance.add_template(1, Template::default());
            instance = instance.add_template(2, Template::default());
            instance =  instance.add_group(
                100,
                TemplateGroupBuilder::default()
                    .add_member(10, 1)
                    .add_member(20, 2)
                    .add_member(30, 3)
                    .build(),
            );

            assert_eq!(instance.check_group_config_validity(), vec![(100, 30, 3)]);
        }

        fn make_instance() -> TerariumBuilder<usize> {
            TerariumBuilder::default()
        }
    }

    mod terarium {
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

        fn make_instance() -> Terarium<String> {
            let mut builder = TerariumBuilder::default();
            builder = builder
                .add_template(
                    "template_a".to_owned(),
                    Template::default()
                        .content_builder()
                        .add_content("template_a cs {{name}}".to_owned(), vec!["cs".to_owned()])
                        .add_content("template_a en {{name}}".to_owned(), vec!["en".to_owned()])
                        .build(),
                );
            builder = builder.add_template(
                "template_b".to_owned(),
                Template::default()
                    .content_builder()
                    .add_content("template_b en {{surname}}".to_owned(), vec!["en".to_owned()])
                    .build(),
            );
            builder = builder.add_group(
                "group_a".to_owned(),
                TemplateGroupBuilder::default()
                    .add_member("A".to_owned(), "template_a".to_owned())
                    .add_member("B".to_owned(), "template_b".to_owned())
                    .build(),
            );
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
