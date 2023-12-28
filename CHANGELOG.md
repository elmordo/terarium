Changelog
=========

## 0.2

* Simplification of the `Template` struct api (when content is added, it cannot be changed).
* `Template.add_content()` now return `Result<Self, TemplateError>`.
* Content can be named (and thus it can be referenced in other templates).
* `ContentBuilder` struct removed.
* The language assignment to `Content` must be unique in one `Template`.
* Template, language and other keys are not generic anymore. All keys are `String` now.
* Attempt to define invalid template group now return `Result::Err` and `Terarium::check_group_config_validity` 
is removed from interface.
* `TerariumBuilder::add_template()` now return `Result<Self, TerariumBuilderError>` to get symmetry with
the `TerariumBuilder::add_group()` method.

## 0.1

* First library release.
