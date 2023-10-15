use std::collections::HashMap;
use std::hash::Hash;

use crate::Template;

pub struct Terrarist {}


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

        fn make_instance() -> TerraristBuilder<usize, usize, usize, usize> {
            TerraristBuilder::default()
        }
    }
}
