# API et MCP

## API Rust aujourd'hui

Le package expose `gotoo_vid2txt::pipeline::Pipeline` et les traits `Acquisition`, `AudioDecoder`, `Transcriber`. L'interface est une alpha non gelée.

## Export de fichier aujourd'hui

- TXT brut.
- JSON `schema_version = 1`, `backend`, `language`, `text`, `segments`, `warnings`.
- Les segments restent vides pour le premier adaptateur Whisper CLI (mode TXT), ne pas inventer les horodatages.

## Préparer MCP sans le coder prématurément

Futur binaire `gotoo-vid2txt-mcp` via le SDK officiel Rust `rmcp`, transport `stdio`, avec par exemple :

1. `vid2txt_capabilities` : backends disponibles, modèles vérifiés, formats.
2. `vid2txt_transcribe` : requête explicitement bornée, résultat avec ID/artefact.
3. `vid2txt_job_status` et `vid2txt_cancel` : **uniquement si** support de tâches asynchrones effectivement implémenté.

Contrat distinct de l'API Rust : JSON Schema versionné avec champs validés, limites et politique d'autorisations. Éviter de transformer les erreurs de sous-processus en chaînes de texte non filtrées destinées au LLM.

## HTTP : après MCP local, si besoin réel

Exposition réseau conditionnée à une authentification, autorisation, protections SSRF, upload quotas, taille de réponse, isolation et journaux sans secrets. On n'ouvre pas de port en v0.1.
