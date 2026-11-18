# Configurer le pipeline

**Statut : candidate alpha non validée par compilation.** La sélection des backends est implémentée dans `config.rs` et `registry.rs`. Aucun chargement de plugins dynamiques.

## Ordre de priorité

1. Valeurs par défaut compilées dans Rust (`auto / ffmpeg / whisper_cpp / txt`).
2. Contenu du fichier passé explicitement avec `--config chemin.toml`.
3. Valeurs individuelles de la CLI (`--audio`, `--exporter`, etc.) : elles gagnent toujours.

Aucun fichier de configuration n'est recherché implicitement (afin de rendre les exécutions reproductibles). Les clés inconnues et les identifiants de backend non implémentés sont des **erreurs bloquantes**, sans repli silencieux.

Exemple complet : `examples/gotoo-vid2txt.toml`.

```toml
[profile.default]
source = "auto"
audio = "ffmpeg"
transcriber = "whisper_cpp"
exporter = "txt"

[backends.whisper_cpp]
binary = "whisper-cli"
model = "models/ggml-base.bin"
```

**Les chemins relatifs** sont actuellement résolus relativement au répertoire de travail du processus, et non au fichier TOML. Dans un contexte automatisé, préférer les chemins absolus.

## Pipeline sans configuration

```bash
gotoo-vid2txt transcribe video.mp4 \
  --model /models/ggml-base.bin --output transcription.txt
```

## Remplacer un composant à la volée

```bash
# Un WAV déjà normalisé (RIFF canonique, 16 kHz, mono, PCM 16 bits).
# Pas de FFmpeg exécuté par ce pipeline.
gotoo-vid2txt transcribe extrait.wav \
  --source local --audio wav-pcm16 --exporter json \
  --model /models/ggml-base.bin --output extrait.json

# Activer l'export TXT même si le fichier de config demande JSON.
gotoo-vid2txt transcribe extrait.wav --config pipeline.toml \
  --exporter txt --output extrait.txt
```

Le backend WAV natif ne sait pas extraire une bande-son MP4, MKV ou WebM. Ce n'est pas un remplaçant de Symphonia ou FFmpeg : il sert aux WAV déjà prêts et à l'évaluation du contrat d'abstraction.

## Dry-run et doctor

```bash
gotoo-vid2txt transcribe video.mp4 --config pipeline.toml --dry-run
# Aucun fichier de sortie créé, aucun média chargé, aucun binaire externe lancé.

gotoo-vid2txt doctor --config pipeline.toml --input video.mp4
# Vérifie les exécutables sélectionnés et la présence du modèle.
# Ne valide ni la qualité du modèle ni un téléchargement YouTube réel.
```

Le `dry-run` ne valide pas actuellement l'existence du modèle ni les permissions du chemin de sortie : c'est une **prévisualisation du routage**, pas une preuve de capacité ou un contrôle exhaustif avant exécution.

## Backends réellement disponibles

| Étape | Identifiant TOML | Disponible | Limite |
|---|---|---|---|
| Acquisition | `auto` / `local` / `yt_dlp` | Oui | URL prise en charge par l'outil externe uniquement |
| Audio/normalisation | `ffmpeg` | Oui | Processus externe |
| Audio/normalisation | `wav_pcm16` | Oui | RIFF WAV canonique déjà 16 kHz mono PCM16 |
| Transcription | `whisper_cpp` | Oui, sous réserve des outils | Exécution CLI et modèle compatible |
| Export | `txt` / `json` | Oui | Pas de SRT/VTT sans segments fiables |
| Audio | `symphonia` | **Non** | Candidat futur, intégration non implémentée |
| Transcription | `candle` | **Non** | Candidat futur, non implémenté |
| Intégration | `mcp` | **Non** | Interface future après contrôle de sécurité |

## Normalisation séparée ?

Le contrat `AudioDecoder::normalize` inclut temporairement décodage et normalisation vers un WAV commun. On ne crée pas un faux module `normalizer` configurable qui ne ferait rien : la séparation sera justifiée et testée lorsqu'un deuxième décodeur exigera des formats audio négociables. Cela évite une abstraction prématurée tout en gardant l'étape audio remplaçable.

## Sécurité et ressources

En cas de backend incompatible ou inconnu, **échec explicite**. Le moteur n'essaie jamais arbitrairement un autre outil. Ne pas fournir d'URL de sources sensibles à des agents non approuvés : contrôles SSRF, quotas, timeouts, sandbox et mécanismes d'annulation restent à implémenter avant tout serveur réseau/MCP.
