# Architecture : pipeline Rust configurable

```text
  CLI                Rust library API         [futur MCP stdio]
   |                        |                          |
   +-------------------- Pipeline ---------------------+
                            |
                          Config
                     (TOML + overrides)
                            |
                        Registry Rust
                            |
       +--------------------+---------------------+-----------------+
       |                    |                     |                 |
 Acquisition           AudioDecoder         Transcriber         Exporter
 local / yt-dlp        FFmpeg / WAV natif     Whisper.cpp          TXT / JSON
       |                    |                     |                 |
       +--------------------+---------------------+-----------------+
                            |
                       OutputTargets
                    staging et publication
```

## Contrats publics de la candidate

- `Config` + `Overrides` : configuration strictement typée ; seules les implémentations enregistrées sont configurables.
- `registry::build` : construit un `Pipeline` avec les adaptateurs sélectionnés. Aucun chargement de DLL, aucun repli silencieux.
- `Source` : distingue URL HTTP(S) et fichier local.
- `Acquisition::acquire` : produit un fichier local.
- `AudioDecoder::normalize` : produit un PCM WAV 16 kHz mono pour la transcription actuelle.
- `Transcriber::transcribe` : retourne un `Transcript` typé, versionné et indépendant de la bibliothèque d'inférence.
- `Exporter::render` : produit des octets (TXT ou JSON), sans écrire dans le système de fichiers.
- `OutputTargets` : contrôle les destinations, puis staging et publication.

## Invariants

1. Les backends ne choisissent pas eux-mêmes les chemins de sortie définitifs.
2. Les adaptateurs ne construisent jamais une ligne shell à partir de l'URL utilisateur.
3. Les étapes propagent les erreurs et le pipeline mesure la durée de chaque phase.
4. Inconnu/incompatible = erreur bloquante ; pas de fallback implicite.
5. `Transcript.schema_version` change si le format de sortie devient incompatible.
6. Les tests doivent démontrer l'injection d'un transcripteur ou exporter alternatif sans modification de l'orchestrateur.
7. Le profil par défaut stable ne doit pas pointer vers un backend expérimental inexistant.

## Limites connues

- Décodage + normalisation restent volontairement groupés sous `AudioDecoder` ; une séparation future requiert un consommateur concret et un test de capacité.
- Le WAV natif est un chemin strict zéro-transcode réservé au PCM 16 kHz mono canonique. Il ne traite pas les vidéos MP4/MKV.
- `Transcript.segments` reste vide pour Whisper.cpp CLI TXT. Pas de SRT/VTT tant que des timestamps fiables ne sont pas disponibles.
- Pas d'annulation, timeout robuste, quotas disque/temps, streaming, SSRF sandbox ou serveur MCP.
- La paire WAV + TXT/JSON n'est pas une transaction atomique ; `--force` n'est pas une garantie totale contre les courses ou coupures brutales.

## Décision : un seul crate

`lib.rs` et `main.rs` appartiennent à **un même package Cargo**, sans workspace multi-crates prématuré. Une extraction de sous-crates pourra se justifier quand la réutilisation externe ou les dépendances MCP/ML le nécessiteront.

Références : [configuration](configuration.md), [ADR 0003](adr/0003-runtime-backend-selection.md).
