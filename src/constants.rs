use serde::Deserialize;

use crate::expr::{evaluate, EvalError};
use crate::functions::Functions;
use crate::value::{Context, Primitive};

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Constants {
    #[serde(rename = "constant")]
    pub constants: Vec<Constant>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Constant {
    pub name: String,

    #[serde(default, rename = "type")]
    pub type_: Option<String>,

    #[serde(rename = "value")]
    value_string: String,

    #[serde(skip)]
    resolved_value: Option<Primitive>,
}
impl Constant {
    pub fn value(&self) -> Primitive {
        self.resolved_value.clone().expect("Value not resolved")
    }

    pub fn resolve_value(&mut self, ctx: &Context) -> Result<(), EvalError> {
        self.resolved_value = Some(evaluate(&self.value_string, ctx, &Functions::default())?);
        Ok(())
    }
}
