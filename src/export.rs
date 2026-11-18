//! Independent, testable transcript rendering backends.
use crate::{domain::Transcript, Result};

pub trait Exporter: Send + Sync {
    fn id(&self) -> &'static str;
    fn extension(&self) -> &'static str;
    fn render(&self, transcript: &Transcript) -> Result<Vec<u8>>;
}

pub struct PlainText;
impl Exporter for PlainText {
    fn id(&self) -> &'static str { "txt" }
    fn extension(&self) -> &'static str { "txt" }
    fn render(&self, transcript: &Transcript) -> Result<Vec<u8>> { Ok(transcript.text.as_bytes().to_vec()) }
}

pub struct Json;
impl Exporter for Json {
    fn id(&self) -> &'static str { "json" }
    fn extension(&self) -> &'static str { "json" }
    fn render(&self, transcript: &Transcript) -> Result<Vec<u8>> { Ok(serde_json::to_vec_pretty(transcript)?) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn both_exporters_share_the_same_schema() {
        let t = Transcript::new("fake", Some("fr".into()), "bonjour\n".into());
        assert_eq!(PlainText.render(&t).unwrap(), b"bonjour\n");
        let json: serde_json::Value = serde_json::from_slice(&Json.render(&t).unwrap()).unwrap();
        assert_eq!(json["schema_version"], 1);
        assert_eq!(json["text"], "bonjour\n");
    }
}
