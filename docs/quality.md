# Qualité et validation

## Matrice de tests attendue avant la première release recommandée

| Niveau | Exemple | Obligatoire ? |
|---|---|---|
| TU domaine | Source, transcript versionné, sérialisation, validations | oui |
| TU sortie | no-clobber, force, symlink, hardlink, alias, I/O fault | oui |
| Contrat backend | remplacement par un fake différent sans toucher `Pipeline` | oui |
| Intégration fake | ordre des phases, sorties, erreurs chaque étape, nettoyage | oui |
| Intégration FFmpeg | 16k/mono/PCM, entrée courte, vidéo sans audio | oui |
| Intégration Whisper | fichier de parole connu, silence, langue explicite et auto | oui |
| Intégration URL | vrai téléchargement (réseau), yt-dlp actuel | oui, job séparé |
| OS | Linux natif et WSL ; Windows/macOS après validation explicite | selon support annoncé |
| Security | liens, dossiers temporaires, sortie partielle, injection, secrets | oui |
| Performance | RTF, RAM, warm/cold, charge représentative | oui, baseline |
| Agent/MCP | sandbox chemins, URL, isolation `stdout`, consentement, annulation | avant MCP |

## Commandes de validation locale

```bash
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps
cargo bench --bench serialization --bench config
mdbook build
```

Le `Cargo.lock` doit être généré et versionné avant la CI `--locked` :

```bash
cargo generate-lockfile
git add Cargo.lock
```

Exécuter aussi, une fois le binaire compilé sur Linux/WSL :

```bash
bash scripts/qa_cli_linux.sh "$(pwd)/target/release/gotoo-vid2txt"
```

Ce script utilise de faux exécutables et ne prouve pas la qualité de transcription.

## Gates non négociables

- Aucun contournement de test, Clippy ou lint pour passer la CI.
- Toute régression reproduite doit devenir un test.
- Les tests mock **ne prouvent pas** l'inférence réelle.
- En cas d'échec non expliqué, statut **BLOCKED**, jamais PASS.
- Un rapport de vérification indique le SHA du commit, les commandes exactes, les résultats et les limites.
