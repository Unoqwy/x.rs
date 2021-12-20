use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(default)]
pub struct RootConfiguration {
    #[serde(flatten)]
    pub mode: RootMode,

    pub hoist: Vec<HoistDeclaration>,
    pub scripts: BTreeMap<String, ScriptDeclaration>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "mode", content = "mode-opts", rename_all = "kebab-case")]
pub enum RootMode {
    Standalone,

    #[serde(rename_all = "kebab-case")]
    Passthrough {
        cmd: String,
        cwd: Option<String>,
        #[serde(default)]
        prepend_args: Vec<String>,
        #[serde(default)]
        append_args: Vec<String>,
    },
}

impl Default for RootMode {
    fn default() -> Self {
        Self::Standalone
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum HoistDeclaration {
    Directory {
        #[serde(alias = "dir")]
        directory: String,
    },
}

#[derive(Clone, Debug, Deserialize)]
pub struct ScriptDeclaration(pub Vec<ScriptCmd>);

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ScriptCmd {
    pub cmd: String,
    pub cwd: Option<String>,
    #[serde(default)]
    pub process_args: BTreeMap<u16, ProcessArgument>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum ProcessArgument {
    Transform { transform: String },
}
