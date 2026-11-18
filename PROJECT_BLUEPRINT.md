# gotoo_vid2txt - Architecture cible et plan de qualification

**Date : 2026-11-18**  
**État : architecture cible + phase 1 configurable implémentée en candidate non compilée.**  
**Dépôt de destination :** https://github.com/gotoo77/gotoo_vid2txt (constaté vide au moment de la préparation).  
**Verdict de publication :** NO-GO pour une release stable ; premier push expérimental seulement après validation locale et décision explicite du propriétaire.

## 1. Vision produit

`gotoo_vid2txt` transforme des vidéos locales ou des contenus réseau pris en charge par un extracteur en transcriptions. Le produit doit fonctionner comme CLI, bibliothèque Rust, puis éventuellement comme service pour agents via MCP. La transcription est **locale par défaut**. Les moteurs externes/distant ne sont activés qu'explicitement.

Fonctions de la cible à long terme :
- Acquisition : fichier local, yt-dlp ; autres connecteurs uniquement lorsqu'un consommateur existe.
- Audio : FFmpeg pour la chaîne généraliste ; WAV PCM16 natif strict pour prouver le remplacement. Symphonia est une évolution candidate, non intégrée.
- Transcription : connecteur whisper.cpp initial, puis connecteurs interchangeables, `whisper-rs`, `transcribe-rs` ou API distantes avec consentement.
- Sorties : TXT, JSON versionné ; SRT/VTT quand les timestamps proviennent réellement du backend.
- Exposition : API Rust immédiate, MCP `stdio` une fois le cœur fiable, HTTP éventuel en dernier.

## 2. Stratégie d'architecture

### Décision centrale

**Un crate/package Cargo avec `src/lib.rs` et `src/main.rs`, en modules internes**. Pas de workspace multi-crates pour le prestige, pas de plugin ABI natif, pas de framework de jobs sans consommateur. Si l'ajout MCP entraîne des dépendances très différentes ou si le core doit être publié indépendamment, extraire des crates après preuve.

```text
User -> CLI / Rust API / [later MCP stdio] / [later HTTP]
                 |
              Pipeline
      +----------+-----------+-------------+
      |          |           |             |
  Acquisition  Audio      Transcriber    Publisher
      |          |           |             |
    Local      FFmpeg   whisper.cpp CLI  TXT, JSON
    yt-dlp                future X        later SRT
```

### Interfaces stables à tester

- `Acquisition::acquire(Source, scratch) -> local Path`
- `AudioDecoder::normalize(local input, output WAV)` ; choix FFmpeg ou WAV PCM16 strict en phase 1
- `Transcriber::preflight`, `Transcriber::protected_paths`, `Transcriber::transcribe(audio, language, scratch) -> Transcript`
- `Transcript` indépendant du backend : `schema_version`, `backend`, `language`, `text`, `segments`, `warnings`
- `Exporter::render(Transcript) -> Vec<u8>` ; TXT et JSON effectivement interchangeables.
- `Config` + `Overrides` + `registry::build` : sélection TOML et CLI, erreurs sur backend inconnu, aucun fallback implicite.
- Orchestration et sorties indépendantes du connecteur utilisé.

**Angle mort explicitement reconnu :** contrat audio WAV fixe. Le deuxième vrai backend pourra nécessiter un format différent. Introduire une négociation de formats/capabilities seulement à ce moment-là, sur tests.

### Adaptateurs futurs (candidats, non intégrés)

| Backend | Avantage potentiel | Vigilance |
|---|---|---|
| `whisper.cpp` CLI | Premier backend stable et isolé en processus | Parsing des fichiers, exécutable externe |
| `whisper-rs` | Inférence embarquée depuis Rust | Build natif, modèles, GPU |
| `transcribe-rs` | Multi-moteur Whisper/ONNX/distant | Transitivité de dépendances, API/ABI, évolution des versions |
| API distante | Facilité sur matériel limité | Envoi de données, coût, consentement, politique de secrets |

Ne pas ajouter tous les backends pour « remplir la grille ». Le second est sélectionné après benchmark de qualité/performance et analyse des contraintes réelles.

## 3. Stratégie de tests

**TU :** modèle de données, validation chemins, synthèse de résultat, commandes externes, erreurs.

**Tests de contrat :** FakeAcquisition/FakeDecoder/FakeTranscriber ; un autre backend fictif doit pouvoir produire une transcription sans modification du pipeline. Tests de failure injection à chaque phase et de non-publication partielle.

**Intégration déterministe :** faux exécutables `yt-dlp`, `ffmpeg`, `whisper-cli` ; lancer le vrai binaire, inspecter arguments, permissions et résultats, sur plateformes annoncées.

**Intégration réelle :** FFmpeg local ; Whisper sur clips autorisés (voix FR/EN, silence, bruit), URL publique accessible ; ne pas confondre mock avec inférence.

**Adversarial :** symlink, hardlink, aliases via `..`, collision TXT/WAV, chemins protégés, quota/espace disque, crash de sous-processus, URL malformées, journaux de secrets, signaux, concurrence.

**Gate :** pas de release si P0 ouvert ; P1 doit être corrigé ou une limitation fortement délimitée doit être explicitement acceptée, sans prétendre supporter le cas concerné.

## 4. Performances

**Micro** via Criterion : sérialisation et parsing TOML. **Macro** : mesures réelles par étape plus RTF, RSS, stockage temporaire, WER/CER sur corpus de référence. Indiquer version du backend, poids du modèle, machine, CPU/GPU, système, threads et SHA des corpus. Une mesure sur un runner GitHub partagé ne doit pas devenir un seuil CI de performance absolu.

## 5. Documentation

- `README.md` : installation et quickstart uniquement.
- `docs/` en Markdown, générée par **mdBook** (plus approprié à une base Rust que Sphinx par défaut).
- `rustdoc` : API Rust, docs doctests dès que les exemples sont compilables.
- `docs/adr/` : décisions, alternatives et motifs.
- `AGENTS.md` : discipline pour Codex/Claude et contributeurs automatisés.
- CI : `cargo fmt`, Clippy strict, tests, rustdoc warnings et mdBook.

## 6. MCP et API

La future API agentique doit réutiliser la bibliothèque, ne pas détourner l'exécutable CLI comme dépendance métier. Pour la première version MCP : **transport stdio**, SDK `rmcp`, pas de port réseau. C'est un autre boundary de sécurité : allowlist de racines, stratégie explicite pour URL, quotas, annulation, contraintes de taille et séparation stdout JSON-RPC/stderr diagnostique.

Le CLI ne peut pas traiter n'importe quel fichier au sens absolu : seulement ceux décodés par FFmpeg et les URL gérées par l'acquisition, accessibles légitimement et sans DRM.

## 7. Phases de livraison

| Phase | Travail | Sortie vérifiable |
|---|---|---|
| M0 | ADR, structure, contrats et tests fake | modularité observable |
| M1 | CLI/Whisper/FFmpeg/yt-dlp, configuration et sélecteurs, WAV natif, TXT/JSON, vrais tests de base | compilation + vraie vidéo locale |
| M2 | Timeout/cancellation/process groups, budgets disque/temps, diagnostics | suite adversariale verte |
| M3 | corpus qualité, baseline perf, matrice OS, docs publiées | GO release initiale |
| M4 | backend alternatif réel | aucun changement au pipeline nécessaire |
| M5 | MCP stdio sécurisé | client MCP réel vérifié |
| M6 | HTTP si usage avéré | sécurité réseau démontrée |

## 8. Limites du livrable présent

Un squelette **avec de l'implémentation réelle** est fourni, mais n'a pas été compilé : Cargo/rustc ne sont pas disponibles dans l'environnement où il a été préparé. Les tests sont présents dans le dépôt fourni, pas exécutés. La CI exige un `Cargo.lock` encore à générer sur une machine avec Cargo.

Par rapport à l'audit précédent, quelques problèmes sont traités conceptuellement et dans le code : dossier temporaire via `tempfile`, publication par staging et remplacement d'entrée (pas de troncature de hardlinks), protection symlink et aliases, `yt-dlp --ignore-config`, séparation pipeline/backends. **Reste à prouver** : absence de régression à l'exécution, comportement multi-OS, limites de durée/taille, terminaison complète de l'arbre des sous-processus, traces de performance, intégration URL actuelle.

Aucune modification ni aucun push n'a été effectué dans le dépôt GitHub pendant la préparation du livrable.

## Addendum phase 1 (2026-11-18)

Voir `VALIDATION_STATUS.md`, `docs/configuration.md` et `docs/adr/0003-runtime-backend-selection.md` pour la configuration réellement implémentée et les limites. Les premiers backends alternatif audio et export existent, mais ni Symphonia ni Candle ne sont intégrés. Les tests sont déclarés, **pas exécutés**.
