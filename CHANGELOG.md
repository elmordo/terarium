Changelog
=========

## 0.2

* Simplification of the `Template` struct api (when content is added, it cannot be changed).
* `Template.add_content()` now return `Result<Self, TemplateError>`.
* Content can be named (and thus it can be referenced in other templates).
* `ContentBuilder` struct removed.
* Used language assignment must be assigned again in one template.
* Template, language and other keys are not generic anymore. All keys are `String` now.

## 0.1

* First library release.
