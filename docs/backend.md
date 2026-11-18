# Écrire un backend de transcription

`src/transcription/mod.rs` définit l'interface. Un moteur n'a pas à connaître yt-dlp, FFmpeg, la CLI ou le système de publication.

```rust,ignore
pub struct ExampleBackend;

impl Transcriber for ExampleBackend {
    fn id(&self) -> &'static str { "example" }

    fn transcribe(
        &self,
        audio: &std::path::Path,
        language: &str,
        scratch: &std::path::Path,
    ) -> gotoo_vid2txt::Result<Transcript> {
        let _ = (audio, language, scratch);
        Ok(Transcript::new("example", None, "hello".into()))
    }
}
```

Si le backend possède des fichiers protégés (modèle, config contenant des secrets), surcharger `protected_paths()`. Si le backend est distant, la politique de consentement, les secrets et la confidentialité doivent être définis avant d'implémenter la transmission de données.

**Contrat de validation du deuxième backend** : test de remplacement sans modifier le pipeline, cas positif, erreurs, transcription vide, langue, horodatages quand fournis, absence de fuite de secrets, consommation ressources et identification explicite de la provenance du modèle.

**Candidats à évaluer, et non intégrés actuellement** : `whisper-rs`, `transcribe-rs` (ONNX/Whisper/moteurs distants), autres APIs de transcription. Préférer un adaptateur étroit autour d'une bibliothèque existante plutôt qu'un second framework interne. Vérifier leurs licences, versions, MSRV, ABI native et accélérations matérielles.
