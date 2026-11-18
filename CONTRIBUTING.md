# Contribuer à gotoo_vid2txt

Les contributions sont bienvenues quand elles comprennent une justification de scope, un test de régression ou de contrat et une validation explicite.

- Ouvrir une issue pour une nouvelle famille de moteurs ou une fonctionnalité qui modifie les contrats.
- Modifier le composant concerné plutôt que le pipeline sans justification.
- Documenter la surface nouvelle et les limites avec des preuves : tests fake != vraie transcription.
- Exécuter fmt, tests, Clippy strict et rustdoc ; inclure le SHA et le résultat dans la PR.
- Les changements du format JSON `schema_version` exigent des tests de compatibilité et un plan de migration.
- Les backends cloud doivent être explicitement opt-in, ne jamais journaliser de tokens ni envoyer implicitement une vidéo privée.
