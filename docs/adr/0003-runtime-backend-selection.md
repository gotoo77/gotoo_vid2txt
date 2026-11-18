# ADR 0003 : sélection explicite des backends au runtime

Statut : **implémentation candidate, non validée par compilation**.

## Contexte

Le pipeline initial avait des traits pour acquisition, audio et transcription, mais la CLI imposait yt-dlp/FFmpeg/Whisper.cpp. L'utilisateur doit pouvoir remplacer un composant sans changer les autres et sans que la configuration mente sur les capacités existantes.

## Décision

- Créer `Config` (`serde` + TOML), valeurs par défaut déclarées, clés inconnues refusées.
- Définir des identifiants de backends sous forme d'enums. Un identifiant inconnu est rejeté.
- Utiliser une registry compilée (`registry.rs`) pour connecter les implémentations aux traits.
- Ajouter un trait `Exporter` distinct, avec deux implémentations véritables.
- Ajouter un deuxième chemin audio réellement implémenté : WAV PCM16 strict, sans FFmpeg.
- Conserver Whisper.cpp comme seul connecteur de transcription **productif**, tout en testant un moteur injecté alternatif sans changer l'orchestrateur.
- Résoudre la config avec l'ordre defaults, TOML explicite, CLI.
- Pas de plugin dynamique, pas de fallback automatique caché, pas de façade Symphonia/Candle fictive.

## Renoncements conscients

- Ne pas prétendre supporter toute vidéo sans FFmpeg.
- Ne pas séparer pour l'instant décodage et rééchantillonnage en contrats indépendants, puisque la sortie WAV 16 kHz mono est commune.
- Ne pas exposer de port HTTP/MCP tant que les limites de ressources et la sécurité URL ne sont pas qualifiées.
- Pas d'ajout de backend de transcription fictif au binaire public : doubles de tests uniquement.

## Tests exigés

Contrats TOML, sélection et CLI override, refus de configuration invalide, parcours WAV sans FFmpeg installé, succès TXT et JSON, fallback absent, doctor sélectionné, échecs d'acquisition/audio/transcription/export et préservation des sorties.
