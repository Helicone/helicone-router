use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(TS, Serialize, Deserialize, Debug, Clone)]
#[ts(export)]
#[serde(tag = "_type")]
pub enum MessageTypeTX {
    Heartbeat,
    RequestConfig {},
    SendLog {
        log: String, // TODO: replace with log
    },
}

#[derive(TS, Serialize, Deserialize, Debug, Clone)]
#[ts(export)]
#[ts(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct AuthConfig {
    pub user_id: String,
    pub organization_id: String,
}

#[derive(TS, Serialize, Deserialize, Debug, Clone)]
#[ts(export)]
#[ts(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct Key {
    pub key_hash: String,
    pub owner_id: String,
}

#[derive(TS, Serialize, Deserialize, Debug, Clone)]
#[ts(export)]
#[ts(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct Config {
    pub auth: AuthConfig,
    pub keys: Vec<Key>,
    pub router_id: String,
    pub router_config: String, // TODO: replace with router config
}

impl Config {
    pub fn get_key_from_hash(&self, key_hash: &str) -> Option<&Key> {
        self.keys.iter().find(|k| k.key_hash == key_hash)
    }
}

#[derive(TS, Serialize, Deserialize, Debug, Clone)]
#[ts(export)]
pub enum Update {
    AuthConfig { data: AuthConfig },
    Config { data: Config },
    Keys { data: Vec<Key> },
}

#[derive(TS, Serialize, Deserialize, Debug, Clone)]
#[ts(export)]
#[derive(Default)]
pub enum Status {
    #[default]
    Success,
    Error {
        message: String,
    },
}

#[derive(TS, Serialize, Deserialize, Debug, Clone)]
#[ts(export)]
pub enum Ack {
    Heartbeat(Status),
    SendLog(Status),
}

#[derive(TS, Serialize, Deserialize, Debug, Clone)]
#[ts(export)]
#[serde(tag = "_type")]
pub enum MessageTypeRX {
    Ack(Ack),
    Update(Update),
    Config { data: String },
    Message { data: String },
}

/// To generate the bindings, run:
/// ```bash
/// BINDING_DIR="../../helicone/packages/llm-mapper/router-bindings" cargo test export_types -- --ignored
/// ```
#[cfg(test)]
mod tests {
    use std::{env, ffi::OsStr, fs, io::Write, path::Path};

    use super::*;

    #[test]
    #[ignore]
    fn export_types() {
        fn generate_exports(dir: &Path) -> Option<Vec<String>> {
            let mut exports: Vec<String> = fs::read_dir(dir)
                .ok()?
                .filter_map(Result::ok)
                .filter_map(|entry| {
                    entry
                        .path()
                        .file_stem()
                        .and_then(OsStr::to_str)
                        .map(str::to_owned)
                })
                .filter(|f| f != "index")
                .map(|f| format!("export * from \"./{f}\""))
                .collect();

            exports.sort();

            Some(exports)
        }

        let binding_dir =
            std::env::var("BINDING_DIR").unwrap_or("./bindings".to_string());
        MessageTypeTX::export_all_to(binding_dir.clone()).unwrap();
        MessageTypeRX::export_all_to(binding_dir.clone()).unwrap();

        let manifest_dir =
            env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let bindings_path = Path::new(&manifest_dir).join(&binding_dir);

        if !bindings_path.exists() {
            println!(
                "bindings path does not exist: {}",
                bindings_path.display()
            );
            std::process::exit(1);
        }
        println!("generating bindings");

        // Generate and write exports for bindings/index.ts
        let exports = generate_exports(&bindings_path);
        let Some(exports) = exports else {
            println!("cargo:warning=No exports found for bindings");
            return;
        };
        let index_path = bindings_path.join("index.ts");
        let mut file = fs::File::create(&index_path).unwrap_or_else(|e| {
            panic!("Failed to create {}: {}", index_path.display(), e)
        });
        file.write_all(exports.join("\n").as_bytes())
            .unwrap_or_else(|e| {
                panic!("Failed to write to {}: {}", index_path.display(), e)
            });
        file.flush().expect("Failed to flush file");

        std::process::Command::new("npx")
            .arg("prettier")
            .arg("--write")
            .arg(format!("{}/**/*.ts", binding_dir))
            .output()
            .unwrap();
    }
}
