# Prompt Codex - audit et validation du socle gotoo_vid2txt

Tu travailles dans le dépôt Rust `gotoo77/gotoo_vid2txt`. Le code de départ peut provenir de l'archive `gotoo_vid2txt_candidate.zip` fournie par l'utilisateur. **N'imagine pas que le dépôt est déjà peuplé**. Inspecte son état réel avant toute opération.

## Mission

Obtenir un socle Rust de transcription modulaire et **vérifié**, fondé sur une bibliothèque réutilisable et un CLI. Le projet doit pouvoir accueillir un deuxième moteur de transcription sans modification du pipeline. Le backend réel initial est `whisper.cpp` via `whisper-cli`, avec acquisition local/yt-dlp, normalisation FFmpeg et sorties TXT/JSON.

## Instructions

1. Inspecter `git status --short`, `git log`, les fichiers, `Cargo.toml`, tests, CI, README et ADR. Préserver le travail utilisateur et signaler toute divergence. Ne pas pousser de code sans autorisation explicite.
2. Lire `PROJECT_BLUEPRINT.md`, `AGENTS.md` et le rapport adversarial précédent. Ne pas transformer le projet en workspace multi-crates sans besoin vérifié.
3. Compléter/corriger les interfaces `Acquisition`, `AudioDecoder`, `Transcriber`, le `Pipeline` et le `Publisher`. Isoler la CLI dans `main.rs`, garder l'API Rust dans `lib.rs`.
4. Ajouter un deuxième `Transcriber` factice dans les tests et vérifier qu'aucune modification du pipeline n'est nécessaire pour l'utiliser.
5. Corriger les problèmes observés, dans cet ordre : intégrité des sorties (symlink, hardlink, alias et sauvegarde de l'ancienne version), confidentialité du workspace, déterminisme de yt-dlp, comportements sous erreurs injectées, gestion des limites de ressources/sous-processus avant qualification stable.
6. Les tests doivent couvrir cas heureux **et** non heureux ; pas de mocks trompeurs prétendant valider Whisper réel. Créer les tests de régression pour chaque défaillance.
7. Générer puis committer `Cargo.lock` pour le binaire. Préparer `mdbook build` et `cargo doc` stricts. Garder la doc alignée sur les fonctionnalités réellement disponibles.
8. Ajouter des benchmarks Criterion pertinents pour le code Rust et un protocole E2E reproductible de temps, RAM, débit/RTF et qualité WER/CER. Ne pas bloquer la CI sur des durées absolues de runners partagés.
9. Ne PAS implémenter MCP ou HTTP dans ce lot. Décrire leurs contrats et frontières de sécurité, sans annoncer qu'ils fonctionnent.

## Validation obligatoire

Depuis le HEAD réellement testé :

```bash
cargo fmt --all -- --check
cargo test --locked --all-targets
cargo clippy --locked --all-targets --all-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps
cargo bench --locked --bench serialization
mdbook build
```

Si les prérequis existent : E2E avec une courte vidéo locale autorisée, FFmpeg et whisper-cli réels, puis URL réelle avec yt-dlp. Exécuter les tests adversariaux de l'audit précédent après adaptation consciente aux noms de commandes du nouveau CLI, **sans affaiblir leurs assertions**.

## Critères d'acceptation

- Aucun code métier dans `main.rs`, structure claire et justifiée.
- Le deuxième backend factice passe dans le pipeline sans changer les étapes.
- Les sorties existantes restent intactes après erreur d'acquisition, décodage, transcription ou staging.
- Aucun effet destructif sur source/modèle via symlink/hardlink avec `--force`.
- CI multi-OS verte pour les OS annoncés ; support retiré des promesses si non testé.
- Documentation `mdBook` et `rustdoc` reproductibles ; exemples réels cohérents.
- Pas de contournement de tests/Clippy ; pas de réécriture annexe gratuite.

## Rapport final obligatoire

Résumé compact : fichiers changés et pourquoi ; SHA testé, état Git ; commandes exécutées, résultats précis ; E2E réels ou absence/raison ; risques restants P0/P1 ; décisions qui attendent le propriétaire ; verdict GO/NO-GO pour premier push expérimental et pour release stable. Ne pas annoncer de succès sur des vérifications non exécutées.
