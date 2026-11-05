use super::Action;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Rule {
  pub action: Action,             // on arguments & libraries
  pub os: Option<RuleOs>,         // only on jvm args & libraries
  pub features: Option<Features>, // only on game args: official minecraft launcher features
}

// os
#[derive(Deserialize, Debug)]
pub struct RuleOs {
  pub name: Option<RuleOsName>,
  pub version: Option<String>, // a regex, unique on jvm args "^10\\." // unique on libraries: "^10\\.5\\.\\d$"
  pub arch: Option<String>,    // architecture, unique value "x86"
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RuleOsName {
  Linux,
  Osx,
  Windows,
}
impl RuleOsName {
  pub fn as_str(&self) -> &'static str {
    match self {
      RuleOsName::Linux => "linux",
      RuleOsName::Osx => "macos",
      RuleOsName::Windows => "windows",
    }
  }
}

#[derive(Deserialize, Debug)]
pub struct Features {
  pub is_demo_user: Option<bool>,
  pub has_custom_resolution: Option<bool>,
  pub has_quick_plays_support: Option<bool>,
  pub is_quick_play_singleplayer: Option<bool>,
  pub is_quick_play_multiplayer: Option<bool>,
  pub is_quick_play_realms: Option<bool>,
}
