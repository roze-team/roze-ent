#[path = "../application_config.rs"]
pub mod application_config;

pub type Config = roze_config::ServiceConfigWithApplication<application_config::ApplicationConfig>;

pub fn load(path: impl AsRef<std::path::Path>) -> Result<Config, config::ConfigError> {
    roze_config::load_service_with_application(path)
}
