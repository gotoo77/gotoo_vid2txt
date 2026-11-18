# gotoo_vid2txt

**Statut : phase 1 alpha candidate, non compilée dans l'environnement de préparation. Pas une release production.**

Outil Rust local-first de conversion de médias en texte, conçu autour d'un pipeline de backends remplaçables. Aucune API de transcription distante n'est requise par les implémentations actuelles.

## Pile actuelle et choix

| Étape | Par défaut | Alternatives actuellement fonctionnelles |
|---|---|---|
| Acquisition | `auto` : fichier local ou yt-dlp selon source | `local`, `yt_dlp` explicitement |
| Audio + normalisation | `ffmpeg` | `wav_pcm16` (WAV canonique déjà normalisé) |
| Transcription | `whisper_cpp` (CLI) | Aucune en production, trait remplaçable et testé avec des doubles |
| Export | `txt` | `json` |

Symphonia, Candle, les autres STT et un serveur MCP sont **des propositions futures, non implémentées**. Aucun backend de secours n'est choisi silencieusement.

## Exemple direct

Installer Rust/Cargo, FFmpeg, whisper.cpp (`whisper-cli`) et un modèle Whisper compatible. Pour les URL YouTube, installer yt-dlp avec ses composants officiels requis (notamment son support de runtime JS).

```bash
cargo test --all-targets
cargo build --release
./target/release/gotoo-vid2txt transcribe video.mp4 \
  --model ./models/ggml-base.bin --language fr --output video.txt

./target/release/gotoo-vid2txt transcribe \
  'https://www.youtube.com/shorts/QoPPlTBY4ls' \
  --model ./models/ggml-base.bin --exporter json --output short.json
```

## Sous-titres YouTube prioritaires

Les URL YouTube utilisent désormais les sous-titres existants avant Whisper. Priorité :
pistes manuelles dans la langue demandée, puis pistes automatiques. Si aucune piste
adaptée n'est présente, le pipeline de transcription audio existant prend le relais.
Une erreur réseau, une piste malformée ou un format non pris en charge fait échouer
l'opération au lieu de masquer le problème. Le format de sous-titres initialement
pris en charge est `json3`.

```bash
# Sans modèle ni exécutable Whisper lorsque des sous-titres sont accessibles.
./target/release/gotoo-vid2txt transcribe \
  'https://www.youtube.com/shorts/VIDEO_ID' \
  --yt-dlp-bin "$HOME/.local/bin/yt-dlp-gotoo" \
  --language fr --output captions.txt

# Forcer uniquement les sous-titres ou forcer l'inférence Whisper.
# --transcript-source youtube
# --transcript-source whisper
```

En JSON, le champ `backend` distingue `youtube-manual-captions`,
`youtube-auto-captions` et `whisper-cpp-cli`. Les sous-titres automatiques
peuvent être inexacts. Si `--audio-out` est demandé, le mode auto utilise
Whisper afin de conserver l'export audio demandé. Le mode YouTube forcé
refuse `--audio-out`. Les médias locaux et autres URL conservent leur pipeline
habituel. Le préflight `doctor` reste pour l'instant celui du pipeline Whisper.

## Sélection des modules

`--config` charge explicitement un TOML. Sans ce paramètre, les valeurs par défaut compilées s'appliquent. Les options CLI gagnent sur le TOML, qui gagne sur les valeurs par défaut.

```bash
./target/release/gotoo-vid2txt transcribe video.mp4 \
  --config examples/gotoo-vid2txt.toml --dry-run

./target/release/gotoo-vid2txt doctor \
  --config examples/gotoo-vid2txt.toml --input video.mp4

# Exemple sans FFmpeg : uniquement un WAV PCM16 mono 16 kHz compatible.
./target/release/gotoo-vid2txt transcribe extrait.wav \
  --source local --audio wav-pcm16 --exporter json \
  --model ./models/ggml-base.bin --output extrait.json
```

Guide complet : [Configuration du pipeline](docs/configuration.md). Le `--dry-run` ne lance pas les moteurs et ne garantit donc pas que l'inférence fonctionne.

## Découpage du projet

- `src/domain.rs` : `Source`, `Job`, `Transcript` et segments indépendants du moteur.
- `src/config.rs` : TOML strict, paramètres et précédence.
- `src/registry.rs` : enregistrement explicite des backends disponibles.
- `src/acquisition/` : accès local/URL, avec adaptateur externe yt-dlp.
- `src/audio/` : interface audio, FFmpeg et WAV natif strict.
- `src/transcription/` : interface STT et adaptateur Whisper.cpp.
- `src/export.rs` : contrats de rendu TXT/JSON.
- `src/output.rs` : staging et publication des fichiers.
- `src/pipeline.rs` : orchestration et durées par étape.
- `src/main.rs` : CLI, plan et doctor, sans logique de décodage.
- `tests/` : simulation des services et défaillances.
- `benches/` : microbenchmarks Criterion (pas des benchmarks d'inférence).
- `docs/` : mdBook et ADR.

## Validation et qualité

```bash
cargo generate-lockfile  # une seule fois, puis COMMIT du Cargo.lock pour la CI
cargo fmt --all -- --check
cargo test --locked --all-targets
cargo clippy --locked --all-targets --all-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps
cargo build --locked --release
bash scripts/qa_cli_linux.sh "$(pwd)/target/release/gotoo-vid2txt"
mdbook build
cargo bench --locked --bench serialization --bench config
```

**Important :** ces commandes restent à exécuter sur une installation Rust réelle. Dans l'environnement de génération du code, Cargo et rustc sont absents, et l'accès aux serveurs de téléchargement est impossible. Ne pas désactiver `--locked` dans la CI pour masquer l'absence de lockfile : produire et versionner ce fichier avant le push.

## Sécurité et limites

Les résultats sont publiés via staging temporaire, mais l'export d'une transcription et d'un WAV ne constitue pas une transaction atomique à deux fichiers. La politique `--force`, les liens, les courses concurrentes, les téléchargements volumineux, les timeouts et l'arrêt des sous-processus demandent encore une qualification adversariale complète. Les backends C/C++, Python et JavaScript sont exécutés comme sous-processus plutôt que chargés directement via FFI. Aucun outil ne peut prétendre être un sandbox à ce stade.

À ce jour, seuls fichiers locaux et URL compatibles avec yt-dlp sont visés. Respecter le droit d'accès et les conditions des plateformes.

## Documentation

```bash
cargo doc --no-deps --open
mdbook build
mdbook serve --open
```

Voir [architecture](docs/architecture.md), [configuration](docs/configuration.md), [qualité](docs/quality.md), [sécurité](docs/security.md), [roadmap](docs/roadmap.md) et [ADR 0003](docs/adr/0003-runtime-backend-selection.md).

Licence MIT. Voir [LICENSE](LICENSE).
