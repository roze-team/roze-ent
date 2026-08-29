use serde::Deserialize;

/// Application-owned typed configuration loaded from the top-level `application` section.
/// Secret references are resolved before deserialization and every field is redacted from Debug.
/// This file is preserved by `rozectl ... generate --update`.
#[derive(Clone, Default, Deserialize, veil::Redact)]
#[redact(all, fixed = 12)]
pub struct ApplicationConfig {
    /// Keeps always-on redaction valid before application fields are added.
    #[doc(hidden)]
    #[serde(skip)]
    pub _roze_redaction_marker: std::marker::PhantomData<()>,
}
