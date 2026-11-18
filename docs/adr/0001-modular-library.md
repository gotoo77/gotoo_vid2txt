# ADR-0001 : un package Cargo, bibliothèque et adaptateurs

- **Statut : proposé, non gelé**
- **Contexte :** MVP historique monofichier, plusieurs sources et transcripteurs envisagés.
- **Décision :** `lib.rs` porte le pipeline, interfaces d'acquisition/normalisation/transcription/publication ; `main.rs` ne fait que CLI et assemblage. Un package Cargo unique pour l'instant.
- **Arguments :** interfaces testables, fausses implémentations simples, coût de maintenance réduit, extension possible vers un second binaire MCP.
- **Non-décisions :** API trait object dynamique, plugin ABI, IPC, sandbox de moteurs, configuration générique universelle, workspace multi-crates.
- **Risques :** normalisation WAV fixe et sortie `Transcript` TXT seulement, à réévaluer avec un vrai deuxième backend.
- **Réexamen :** premier backend aux capacités incompatibles ou deuxième consommateur public nécessitant un packaging distinct.
