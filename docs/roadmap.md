# Roadmap et critères de décision

| Étape | Objectif | Critère de sortie |
|---|---|---|
| M0 - Design | Contrats et boundaries testables | ADR, tests de remplacement de backend, projet modulaire |
| M1 - Core alpha | CLI local/URL, FFmpeg, Whisper, TXT/JSON, sécurité sortie | `fmt`/`clippy`/`test` + fake integration + vrai clip local |
| M2 - Fiabilité | Timeouts, limites de taille/durée, diagnostics, arrêt enfants | régressions adversariales, tests injectés et tests longs |
| M3 - Qualification | OS annoncés, corpus réel, WER/CER, perf baseline, docs auto | CI multi-OS et publication documentée |
| M4 - 2e backend | Adaptateur réellement différent (ex. `transcribe-rs`) | ajouter sans toucher au pipeline, tests comparatifs |
| M5 - MCP | `stdio`, contrats structurés, politique restrictive | test avec au moins un vrai client MCP, audit sécurité agentique |
| M6 - HTTP | Seulement sur besoin démontré | modèle de menace, quotas, auth, tests E2E |

**L'alpha candidate présente actuellement dans l'archive n'est ni M1 validé ni release candidate.**

## GO pour premier push public

Le premier commit peut être étiqueté `experimental`, à condition d'annoncer les limites. Pour une première version recommandée à des utilisateurs, exiger :

- Cargo.lock committé, compilation réelle, `fmt/test/clippy/doc` PASS.
- Zéro défaillance P0 ou P1 non traitée et non explicitement requalifiée.
- Médias réels local et URL testés, sorties protégées.
- Exécution des tests adversariaux sous Linux/WSL ; Windows/macOS soit testés, soit déclarés non supportés.
- README et exemples de la version réellement exécutée.
- Pas de promesse d'API/MCP tant que non implémenté.
