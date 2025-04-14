use super::Rule;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Arguments {
  #[serde(default)]
  pub game: Vec<Argument>,
  #[serde(default)]
  pub jvm: Vec<Argument>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Argument {
  Normal(ArgumentValue),
  Ruled(RuledArgument),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ArgumentValue {
  Single(String),
  Many(Vec<String>),
}

#[derive(Deserialize, Debug)]
pub struct RuledArgument {
  pub rules: Vec<Rule>,
  pub value: ArgumentValue,
}

// GAME NORMAL ARGUEMNTS GUIDE //
// Single: (/)
// Many:
// --accessToken ${auth_access_token}
// --assetIndex ${assets_index_name}
// --assetsDir ${assets_root}
// --clientId ${clientid}
// --gameDir ${game_directory}
// --userType ${user_type}
// --username ${auth_player_name}
// --uuid ${auth_uuid}
// --version ${version_name}
// --versionType ${version_type} See VersionType enum
// --xuid ${auth_xuid}

// GAME RULED ARGUEMNTS GUIDE //
// Single:
// --demo
// Many:
// --height ${resolution_height}
// --width ${resolution_width}
// --quickPlayMultiplayer ${quickPlayMultiplayer}
// --quickPlayPath ${quickPlayPath}
// --quickPlayRealms ${quickPlayRealms}
// --quickPlaySingleplayer ${quickPlaySingleplayer}

////////////////

// JVM NORMAL ARGUEMNTS GUIDE //
// Single: (/)
// Many:
// -cp ${classpath}
// -Dio.netty.native.workdir=${natives_directory}
// -Djava.library.path=${natives_directory}
// -Djna.tmpdir=${natives_directory}
// -Dminecraft.launcher.brand=${launcher_name}
// -Dminecraft.launcher.version=${launcher_version}
// -Dorg.lwjgl.system.SharedLibraryExtractPath=${natives_directory}

// JVM RULED ARGUEMNTS GUIDE //
// Single:
//
// Many:
// -Dos.name=Windows 10  //?? win10 only
// -Dos.version=10.0     //?? win10 only
// -XstartOnFirstThread  //?? macos only
