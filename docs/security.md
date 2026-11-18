# Sécurité, confidentialité et futur usage agentique

## CLI local : frontières de confiance

L'utilisateur choisit explicitement la source, les chemins et les programmes externes. yt-dlp et FFmpeg traitent des données potentiellement non fiables. Le backend ne contacte aucune API de transcription distante, mais yt-dlp peut contacter la plateforme d'origine.

- Dossier temporaire créé par `tempfile` et supprimé par RAII après retour ordinaire.
- Publication par staging ; ne jamais ouvrir le chemin final en troncature directe.
- `--force` vérifie et rejette les liens symboliques déjà présents ; une protection parfaite contre les modifications concurrentes de répertoires demanderait un travail spécifique à chaque OS.
- Journaliser les étapes et codes de sortie sans exposer le contenu transcrit, les cookies, l'URL complète ou les jetons.
- Les ressources (temps/disque/RAM) et la descendance des processus externes ne sont pas encore bornées : **bloquant pour un service non supervisé ou public**.

## MCP : le niveau de confiance change

Un agent peut fournir des paramètres contenant des textes non fiables (prompt injection dans métadonnées/transcriptions) et déclencher un téléchargement ou une lecture locale.

Pour un premier MCP `stdio` local, prévoir :

- **Politique opt-in explicite** pour chaque racine autorisée et chaque domaine d'URL.
- Par défaut, **aucun accès arbitraire à `~`, aux documents personnels ou aux secrets**.
- Limites de durée, taille, concurrence, nombre de jobs et taille des sorties ; annulation effective des sous-processus.
- Sortie MCP JSON-RPC exclusivement sur `stdout`, diagnostics sur `stderr`.
- Résultats structurés et bornés, avec possibilité de référence d'artefact, pas d'immenses transcriptions dans le contexte.
- Pas de serveur HTTP exposé sur une interface réseau par défaut.
- Aucun backend distant implicitement choisi pour traiter des médias privés.

Le transport `stdio` n'apporte **pas à lui seul** une sandbox ni l'autorisation. L'hôte MCP doit permettre une configuration restrictive et une confirmation d'action sensible. Ne pas affirmer une intégration universelle à tous les produits IA sans l'avoir testée par client.
