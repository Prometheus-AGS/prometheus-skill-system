// Modified for the Prometheus Skill System vendored optimizer. See PROVENANCE.md.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub description: Option<String>,
    pub prefix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Signature {
    pub name: String,
    pub instruction: String,
    pub inputs: Vec<Field>,
    pub outputs: Vec<Field>,
}

impl Signature {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            instruction: String::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn with_instruction(mut self, instruction: impl Into<String>) -> Self {
        self.instruction = instruction.into();
        self
    }

    pub fn input(mut self, name: impl Into<String>, desc: Option<&str>) -> Self {
        self.inputs.push(Field {
            name: name.into(),
            description: desc.map(|s| s.to_string()),
            prefix: None,
        });
        self
    }

    pub fn output(mut self, name: impl Into<String>, desc: Option<&str>) -> Self {
        self.outputs.push(Field {
            name: name.into(),
            description: desc.map(|s| s.to_string()),
            prefix: None,
        });
        self
    }
}
