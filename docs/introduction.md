# Introduction

`gotoo_vid2txt` vise un socle réutilisable pour la conversion de vidéos locales et distantes en transcriptions. Objectif initial : CLI Rust fiable et auditable, offline pour la transcription, avec un coût de maintenance raisonnable.

**Existant dans la candidate** : binaire Rust, bibliothèque, backend Whisper CLI, yt-dlp/FFmpeg, sortie TXT/JSON, tests avec doubles, benchmark de sérialisation. **Pas encore validé** : compilation, CI, tests réels et stabilité.

**Prévu ensuite** : deuxième moteur réel, formats horodatés, métriques reproductibles, API structurée et serveur MCP `stdio`.

**Vision ultérieure** : service HTTP autorisé, traitement par lots, GPU et optimisations de pipeline. Ce ne sont pas des fonctionnalités actuelles.
