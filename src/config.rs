use std::io::Write;

use crate::models::model::Model;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

fn serialize_model<S>(model: &Option<Model>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match model {
        Some(m) => serializer.serialize_str(&m.code()),
        None => serializer.serialize_none(),
    }
}

fn deserialize_model<'de, D>(deserializer: D) -> Result<Option<Model>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    match s {
        Some(code) => Ok(Some(Model::from_code(&code))),
        None => Ok(None),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(
        serialize_with = "serialize_model",
        deserialize_with = "deserialize_model"
    )]
    pub model: Option<Model>,
}

impl Config {
    pub fn new() -> Self {
        Self { model: None }
    }

    pub fn set_model(&mut self, model: Model) {
        self.model = Some(model);
    }

    pub fn save(&self) {
        let toml = toml::to_string_pretty(&self).unwrap();

        std::fs::write("neura.toml", toml).unwrap();
    }

    pub fn load() -> Self {
        let toml = std::fs::read_to_string("neura.toml").unwrap();

        toml::from_str(&toml).unwrap()
    }

    pub fn is_initialized() -> bool {
        std::path::Path::new("neura.toml").exists()
    }

    pub fn dotenv_exists() -> bool {
        std::path::Path::new(".env").exists()
    }

    pub fn set_env(&self, key: String, value: String) {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(".env")
            .unwrap();

        let line = format!("{}={}\n", key, value);

        file.write_all(line.as_bytes()).unwrap();
    }

    pub fn create_dotenv(&self) {
        std::fs::File::create(".env").unwrap();
    }

    pub fn check_env_var_exists(&self, key: String) -> bool {
        let dotenv = std::fs::read_to_string(".env").unwrap();

        dotenv.contains(&key)
    }
}
