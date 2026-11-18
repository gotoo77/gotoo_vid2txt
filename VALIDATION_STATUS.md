# Validation de la phase 1 : sélection configurable des backends

Date : 2026-11-18. **Statut : NO-GO pour la publication d'une version fiable.**

## Réalisé

- `src/config.rs` : TOML strict, erreurs si clés ou backend inconnus, defaults et CLI overrides.
- `src/registry.rs` : sélection explicite des adaptateurs compilés, validation type de source.
- `src/audio/wav_pcm16.rs` : décodage zéro-transcode de WAV canonique pré-normalisé, strictement validé (44 octets, RIFF/WAVE, mono 16 kHz PCM16, tailles exactes).
- `src/export.rs` : trait d'export et implémentations TXT/JSON.
- `src/pipeline.rs` : exporter injecté comme les autres adaptateurs.
- `src/main.rs` : `transcribe --config`, overrides, `doctor` limité aux backends concernés et `--dry-run` sans exécution.
- Tests Rust : 28 tests déclarés (unitaires et intégration avec doubles), couvrant notamment remplacement de transcripteur, remplacement d'exporteur, backends incompatibles et préservation des anciens outputs sur erreur.
- QA Linux : cas mockés, changement de backend audio et format sans FFmpeg, config invalide, aucune bascule automatique, `doctor`, `dry-run`.
- Critère de performance : benchmark Criterion de sérialisation JSON + parsing TOML (microbenchmarks seulement).
- Docs : README, configuration, ADR 0003, exemple TOML, sommaire mdBook.
- CI : tests, lint, docs, QA mock Linux, OS matrix Win/macOS/Linux. La CI attend un `Cargo.lock` committé.

## Validation réellement exécutée

| Contrôle | Résultat |
|---|---|
| Lecture et inspection du socle Rust original | Effectuée |
| Parsing du `Cargo.toml` via Python `tomllib` | PASS |
| Correspondance des liens Markdown du sommaire mdBook | PASS |
| `bash -n scripts/qa_cli_linux.sh` | PASS |
| Présence de l'étape QA mock Linux dans la CI | PASS |
| `cargo fmt --check` | NON EXÉCUTÉ, outil absent |
| `cargo test` / `clippy` / `rustdoc` | NON EXÉCUTÉS, outils absents |
| QA CLI contre exécutable réellement compilé | NON EXÉCUTÉ, pas de binaire |
| `mdbook build` | NON EXÉCUTÉ, outil absent |
| Benchmarks Criterion | NON EXÉCUTÉS |
| Whisper réel, URL YouTube réelle | NON EXÉCUTÉS |
| `cargo audit`/`cargo deny` | NON EXÉCUTÉS |
| Validation hardware Windows/WSL/macOS | NON EXÉCUTÉE |

Le système d'exécution ne dispose pas de `cargo`, `rustc`, `rustfmt` ou `mdbook`. L'accès direct aux serveurs de téléchargement de Rust échoue (DNS indisponible). Il serait erroné de qualifier la phase 1 de compilée ou de testée.

## Gates obligatoires sur l'environnement Rust de l'utilisateur

```bash
# À la racine du projet décompressé.
cargo generate-lockfile
cargo fmt --all
cargo fmt --all -- --check
cargo test --locked --all-targets
cargo clippy --locked --all-targets --all-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps
cargo build --locked --release
bash scripts/qa_cli_linux.sh "$(pwd)/target/release/gotoo-vid2txt"
cargo bench --locked --bench serialization --bench config
mdbook build
```

`cargo fmt --all` est **une modification de formatage à vérifier**, pas une validation. Le contrôle de formatage qui suit doit réussir. Ne committer/pousser que quand l'ensemble des gates requis passe et que les éventuelles exceptions sont documentées.

## Gates non terminés pour une publication production

- Corriger toutes erreurs de compilation/lint dans cet environnement (si détectées).
- Générer, vérifier et versionner `Cargo.lock`.
- Reprendre le banc adversarial précédent (hardlinks, symlinks, collision de sorties, permissions et races concurrentes) et vérifier la couverture des modifications.
- Limites de durée/taille/ressources et annulation fiable des sous-processus, notamment yt-dlp et Whisper.
- Qualifier une vraie transcription avec corpus français/anglais, WER/RTF et empreinte RAM/VRAM.
- Auditer dépendances et licences avec `cargo audit` et `cargo deny`.
- Qualifier FFmpeg, yt-dlp, Whisper externes et le périmètre de sécurité de l'exécution agentique avant MCP.
- Expérimenter Symphonia/Candle comme *candidats* avant de les enregistrer comme backends disponibles.

## Politique de dépôt

Ce dossier est une **candidate locale**, non poussée vers `gotoo77/gotoo_vid2txt`. L'état GitHub vérifié au début de cette phase est un dépôt public vide. Le premier push ne doit pas être présenté comme une release stable sans les validations requises.
