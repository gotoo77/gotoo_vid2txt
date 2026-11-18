# Prompt pour validation et remédiation avant push

Tu travailles dans le dépôt Rust `gotoo_vid2txt` contenant une candidate alpha du pipeline de transcription configurable.

Mission : inspecter, compiler, corriger **strictement** les défauts empêchant la phase 1 de fonctionner comme spécifiée, puis valider la substitution des backends. Ne pas ajouter Symphonia, Candle, HTTP/MCP ou un autre moteur tant que la phase 1 ne passe pas.

1. Inspecte d'abord `git status --short`, `git branch --show-current`, `Cargo.toml`, `src/`, `tests/`, `scripts/`, `.github/workflows/ci.yml`, `docs/configuration.md` et `VALIDATION_STATUS.md`. Ne suppose pas l'état du dépôt ; STOP si modifications inattendues.
2. Vérifie les contrats : defaults `auto/ffmpeg/whisper_cpp/txt`, TOML strict, CLI override, `--dry-run` sans effet, `doctor` ne vérifiant que la chaîne sélectionnée, absence de fallback invisible.
3. Vérifie que `--audio wav-pcm16` traite réellement un WAV 16 kHz mono PCM16 sans invoquer FFmpeg et rejette un WAV incompatible ; que TXT/JSON sont réellement interchangeables ; qu'un autre `Transcriber` peut être injecté dans `Pipeline` sans toucher à l'orchestrateur.
4. Génère `Cargo.lock` et exécute dans l'ordre :

```bash
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

5. Si une gate échoue, diagnostique l'erreur réelle et corrige le plus petit changement justifié, avec test de régression. Relance la gate concernée puis toutes les gates. **Jamais de désactivation des tests, de `clippy`, de lints ou de règles de CI.** Si outil absent, déclare BLOCKED et n'invente pas un PASS.
6. Vérifie que les docs et exemples décrivent les options qui existent réellement. Ne présente jamais le backend Candle/Symphonia/MCP comme implémenté.
7. Contrôle les erreurs à risque : output symlinks/hardlinks, collisions, traitement URL, sous-processus sans timeout, confidentialité des chemins et logs. S'il reste un bloqueur de sécurité non prévu dans le scope, signale-le explicitement plutôt que de bricoler.
8. Fournis un rapport bref et précis : fichiers changés, motivation, SHA/diff, tests et benchmarks avec leur résultat, limites et verdict GO/NO-GO. **Ne pousse pas et ne crée pas de release sans demande explicite.**
