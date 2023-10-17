use std::collections::HashMap;
use std::hash::Hash;

use tera::{Context, Error as TeraError};
use tera::Tera;
use thiserror::Error;

use crate::Template;

pub struct Terrarist<TemplateKey, LocaleKey, GroupKey, GroupMemberKey>
    where
        TemplateKey: Eq + Hash + Copy,
        LocaleKey: Eq + Hash + Copy,
        GroupKey: Eq + Hash + Copy,
        GroupMemberKey: Eq + Hash + Copy,
{
    tera: Tera,
    template_map: HashMap<TemplateKey, HashMap<LocaleKey, String>>,
    groups: HashMap<GroupKey, HashMap<GroupMemberKey, TemplateKey>>,
}

impl<TemplateKey, LocaleKey, GroupKey, GroupMemberKey> Terrarist<TemplateKey, LocaleKey, GroupKey, GroupMemberKey>
    where
        TemplateKey: Eq + Hash + Copy,
        LocaleKey: Eq + Hash + Copy,
        GroupKey: Eq + Hash + Copy,
        GroupMemberKey: Eq + Hash + Copy,
{
    pub fn build_template(
        &self,
        context: &Context,
        template_key: &TemplateKey,
        locale: &LocaleKey,
        fallback_locale: Option<&LocaleKey>,
    ) -> Result<String, TerraristError> {
        let template = self
            .template_map.get(template_key).ok_or_else(|| TerraristError::TemplateNotFound)?;
        let content_key = template
            .get(locale)
            .or_else(|| {
                fallback_locale.map(|k| template.get(k)).flatten()
            })
            .ok_or_else(|| TerraristError::LocaleNotFound)?;
        Ok(self.tera.render(content_key.as_str(), context)?)
    }

    pub fn build_group(
        &self,
        context: &Context,
        group_key: &GroupKey,
        locale: &LocaleKey,
        fallback_locale: Option<&LocaleKey>,
    ) -> Result<HashMap<GroupMemberKey, String>, TerraristError> {
        todo!()
    }
}


impl<TemplateKey, LocaleKey, GroupKey, GroupMemberKey> Default for Terrarist<TemplateKey, LocaleKey, GroupKey, GroupMemberKey>
    where
        TemplateKey: Eq + Hash + Copy,
        LocaleKey: Eq + Hash + Copy,
        GroupKey: Eq + Hash + Copy,
        GroupMemberKey: Eq + Hash + Copy,
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
pub enum TerraristError {
    #[error("There is no template")]
    TemplateNotFound,
    #[error("Locale not found")]
    LocaleNotFound,
    #[error("There is no group")]
    GroupNotFound,

    #[error("Error when rendering template")]
    RenderingFailed(TeraError),
}


impl From<TeraError> for TerraristError {
    fn from(value: TeraError) -> Self {
        Self::RenderingFailed(value)
    }
}


#[derive(Default)]
pub struct TerraristBuilder<TemplateKey, LocaleKey, GroupKey, GroupMemberKey>
    where
        TemplateKey: Eq + Hash + Copy,
        LocaleKey: Eq + Hash + Copy,
        GroupKey: Eq + Hash + Copy,
        GroupMemberKey: Eq + Hash + Copy,
{
    templates: HashMap<TemplateKey, Template<LocaleKey>>,
    groups: HashMap<GroupKey, HashMap<GroupMemberKey, TemplateKey>>,
}


impl<TemplateKey, LocaleKey, GroupKey, GroupMemberKey> TerraristBuilder<TemplateKey, LocaleKey, GroupKey, GroupMemberKey>
    where
        TemplateKey: Eq + Hash + Copy,
        LocaleKey: Eq + Hash + Copy,
        GroupKey: Eq + Hash + Copy,
        GroupMemberKey: Eq + Hash + Copy,
{
    pub fn add_template(&mut self, key: TemplateKey) -> &mut Template<LocaleKey> {
        let template = Template::default();
        self.templates.insert(key, template);
        self.templates.get_mut(&key).unwrap()
    }

    pub fn get_template(&mut self, key: &TemplateKey) -> Option<&mut Template<LocaleKey>> {
        self.templates.get_mut(key)
    }

    pub fn remove_template(&mut self, key: &TemplateKey) -> Option<Template<LocaleKey>> {
        self.templates.remove(key)
    }

    pub fn add_group(&mut self, key: GroupKey) -> &mut HashMap<GroupMemberKey, TemplateKey> {
        self.groups.insert(key, HashMap::new());
        self.groups.get_mut(&key).unwrap()
    }

    pub fn get_group(&mut self, key: &GroupKey) -> Option<&mut HashMap<GroupMemberKey, TemplateKey>> {
        self.groups.get_mut(key)
    }

    pub fn remove_group(&mut self, key: &GroupKey) -> Option<HashMap<GroupMemberKey, TemplateKey>> {
        self.groups.remove(key)
    }

    pub fn check_group_config_validity(&self) -> Vec<(GroupKey, GroupMemberKey, TemplateKey)> {
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

    pub fn build(self) -> Result<Terrarist<TemplateKey, LocaleKey, GroupKey, GroupMemberKey>, TerraristBuilderError<GroupKey, GroupMemberKey, TemplateKey>> {
        let check_result = self.check_group_config_validity();
        if !check_result.is_empty() {
            return Err(TerraristBuilderError::GroupIntegrityProblem(check_result));
        }

        let mut instance = Terrarist::default();
        let mut tera_template_id: u32 = 1;

        // build templates
        self.templates.into_iter().try_for_each(|(template_key, template)| {
            template.collect_contents().into_iter().try_for_each(|(content, locales)| {
                let template_name = format!("template#{}", tera_template_id);
                tera_template_id += 1;
                instance.tera.add_raw_template(&template_name, &content)?;

                locales.into_iter().for_each(|locale_key| {
                    instance.template_map.entry(template_key).or_default().insert(locale_key, template_name.clone());
                });

                Ok::<_, TerraristBuilderError<_, _, _>>(())
            })?;
            Ok::<_, TerraristBuilderError<_, _, _>>(())
        })?;

        instance.groups = self.groups;
        Ok(instance)
    }
}


#[derive(Debug, Error)]
pub enum TerraristBuilderError<GroupKey, GroupMemberKey, TemplateKey> {
    #[error("Unable to build template")]
    TemplateBuildingError(TeraError),
    #[error("Cannot build template groups - some templates are missing")]
    GroupIntegrityProblem(Vec<(GroupKey, GroupMemberKey, TemplateKey)>),
}


impl<GroupKey, GroupMemberKey, TemplateKey> From<TeraError> for TerraristBuilderError<GroupKey, GroupMemberKey, TemplateKey> {
    fn from(value: TeraError) -> Self {
        Self::TemplateBuildingError(value)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod terrarist_builder {
        use super::*;

        #[test]
        fn add_template() {
            let mut instance = make_instance();
            let template = instance.add_template(1);
            template.add_content("foo".to_string(), vec![1, 2]);

            assert_eq!(instance.templates.len(), 1);
            let template = instance.templates[&1].clone();
            let mut contents = template.collect_contents();
            assert_eq!(contents.len(), 1);
            contents[0].1.sort();
            assert_eq!(contents, vec![("foo".to_string(), vec![1, 2])]);
        }

        #[test]
        fn get_template() {
            let mut instance = make_instance();
            instance.add_template(1);
            assert!(instance.get_template(&1).is_some());
        }

        #[test]
        fn get_not_existing_template() {
            let mut instance = make_instance();
            instance.add_template(1);
            assert!(instance.get_template(&2).is_none());
        }

        #[test]
        fn remove_template() {
            let mut instance = make_instance();
            instance.add_template(1);
            let tpl = instance.remove_template(&1);
            assert!(tpl.is_some());
        }

        #[test]
        fn remove_not_existing_template() {
            let mut instance = make_instance();
            instance.add_template(1);
            let tpl = instance.remove_template(&2);
            assert!(tpl.is_none());
        }

        #[test]
        fn group_manipulation() {
            let mut instance = make_instance();
            {
                let grp = instance.add_group(1);
                grp.insert(1, 1);
            }
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
            instance.add_template(1);
            instance.add_template(2);

            {
                let grp = instance.add_group(100);
                grp.insert(10, 1);
                grp.insert(20, 2);
                grp.insert(30, 3);
            }

            assert_eq!(instance.check_group_config_validity(), vec![(100, 30, 3)]);
        }

        fn make_instance() -> TerraristBuilder<usize, usize, usize, usize> {
            TerraristBuilder::default()
        }
    }
}
