use std::collections::HashMap;
use std::hash::Hash;

use crate::Template;

pub struct Terrarist {}


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
