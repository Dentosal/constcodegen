use std::collections::HashMap;

use serde::Deserialize;

use crate::constants::Constant;
use crate::format_value::*;
use crate::template;

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Options {
    /// Top-level code generation options
    pub codegen: CodegenOptions,

    /// Per-language settings
    lang: HashMap<String, LangOptions>,
}
impl Options {
    pub fn languages(&self) -> Vec<(&String, &LangOptions)> {
        self.lang
            .iter()
            .filter(|(ref name, _)| self.codegen.enabled.contains(name))
            .collect()
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct CodegenOptions {
    /// Languages to generate files for
    enabled: Vec<String>,

    // Comment sections
    #[serde(default)]
    pub comment_sections: bool,
}

/// Options for a single programming language or other data format
/// All templates described here are always followed by a linebreak
#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct LangOptions {
    /// File extension for this language
    pub file_ext: String,

    /// Template for generating a single constant.
    template: String,

    /// Template for importing a dependency.
    /// Dependencies in types are not allowed if this is None.
    #[serde(default)]
    import: Option<String>,

    /// Template for a comment.
    /// Comments are not emitted if this is None.
    #[serde(default)]
    comment: Option<String>,

    /// Template for the start of the constants block
    #[serde(default)]
    intro: Option<String>,

    /// Template for the end of the constants block
    #[serde(default)]
    outro: Option<String>,

    /// Literal formatting
    #[serde(default)]
    format: Format,

    /// Formatter command.
    /// Must accept input from stdin and output formatted code to stdout.
    #[serde(default)]
    pub formatter: Option<Vec<String>>,

    /// Types
    #[serde(default, rename = "type")]
    pub types: HashMap<String, LangTypeOptions>,
}
impl LangOptions {
    /// Returns None if `type` field is required but `None`
    pub fn format_constant(&self, constant: &Constant) -> Option<String> {
        let mut t_ctx = HashMap::new();
        t_ctx.insert("$name", constant.name.clone());
        t_ctx.insert(
            "$value",
            constant
                .type_
                .clone()
                .and_then(|t| self.types.get(&t))
                .map(|t_opts| t_opts.format.clone())
                .unwrap_or_else(|| self.format.clone())
                .format(&constant.value()),
        );

        if template::contains_parameter(&self.template, "$type") {
            let type_ = constant.type_.clone()?;
            t_ctx.insert("$type", type_.clone());
            if let Some(type_opts) = self.types.get(&type_) {
                if let Some(type_name) = &type_opts.name {
                    t_ctx.insert("$type", type_name.clone());
                }

                let old_value = t_ctx["$value"].clone();
                t_ctx.insert(
                    "$value",
                    format!(
                        "{}{}{}",
                        type_opts.value_prefix, old_value, type_opts.value_suffix
                    ),
                );
            }
        }

        Some(template::replace_parameters(&self.template, &t_ctx))
    }

    /// Returns None if the language doesn't support imports
    pub fn format_import(&self, import: &str) -> Option<String> {
        let mut t_ctx = HashMap::new();
        t_ctx.insert("$import", import.to_owned());
        let im = self.import.clone()?;
        Some(template::replace_parameters(&im, &t_ctx))
    }

    pub fn format_comment(&self, comment: &str) -> String {
        let mut t_ctx = HashMap::new();
        t_ctx.insert("$comment", comment.to_owned());
        self.comment
            .clone()
            .map(|c| format!("{}\n", template::replace_parameters(&c, &t_ctx)))
            .unwrap_or_else(String::new)
    }

    pub fn format_intro(&self) -> String {
        let t_ctx = HashMap::new();
        self.intro
            .clone()
            .map(|c| format!("{}\n", template::replace_parameters(&c, &t_ctx)))
            .unwrap_or_else(String::new)
    }

    pub fn format_outro(&self) -> String {
        let t_ctx = HashMap::new();
        self.outro
            .clone()
            .map(|c| format!("{}\n", template::replace_parameters(&c, &t_ctx)))
            .unwrap_or_else(String::new)
    }

    pub fn constant_imports(&self, constant: &Constant) -> Vec<String> {
        if let Some(type_) = constant.type_.clone() {
            if let Some(type_opts) = self.types.get(&type_) {
                return type_opts.import.clone();
            }
        }
        Vec::new()
    }
}

/// Additional formatting for a single type in some language
#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct LangTypeOptions {
    /// Use a different name for the type
    pub name: Option<String>,

    /// Prefix when using a value
    pub value_prefix: String,

    /// Suffix when using a value
    pub value_suffix: String,

    /// Override literal formatting
    pub format: Format,

    /// Requires these dependencies imported to use
    pub import: Vec<String>,
}
