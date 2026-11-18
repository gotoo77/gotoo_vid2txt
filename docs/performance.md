# Performances : protocole reproductible

## Ne pas confondre micro-benchmark et transcription réelle

`cargo bench --bench serialization` mesure uniquement le coût de sérialisation des segments. Il **ne dit rien** de la rapidité de Whisper ou de FFmpeg.

Le futur benchmark end-to-end devra mesurer les mêmes médias, modèle et machine :

- **RTF = temps de transcription / durée audio** (ex. RTF 0,5 = 2x temps réel).
- Durée d'acquisition, normalisation, inférence, publication, totale.
- Débit en minutes audio/minute, pic de RSS, stockage temporaire de pointe.
- WER/CER sur références humaines, à corpus et langue constants.
- 3 runs minimum + warm-up déclaré, médiane et dispersion (IQR, p95 si suffisamment d'échantillons).
- Mode froid/chaud, CPU/GPU, threads, OS, architecture, modèles, versions d'outils, hashes du corpus.

**Interdiction de conclure** à une régression sur une seule mesure ou entre deux matériels différents. CI doit collecter des métriques, pas imposer une limite de temps absolue susceptible de fluctuer avec les runners partagés.

## Corpus minimum

1. Voix française 15-30 s, transcription de référence autorisée.
2. Voix anglaise 15-30 s.
3. Silence 10 s.
4. Musique sans parole (détection hallucinations éventuelles).
5. Vidéo avec audio stéréo / taux d'échantillonnage différent.
6. Entrée longue pour mesurer la pression RAM et l'espace temporaire.

Ne pas versionner par accident des vidéos téléchargées, des pistes dont les droits sont inconnus ou des modèles volumineux.

## Phase 1 : nouveau microbenchmark de configuration

`cargo bench --bench config` mesure le temps de parsing d'un document TOML de sélection de pipeline. `cargo bench --bench serialization` mesure la sérialisation JSON d'une transcription à 500 segments. Ce sont **des microbenchmarks**, et non des mesures d'ASR ou de décodage audio. Les chiffres n'existent pas avant exécution sur une machine identifiée.
