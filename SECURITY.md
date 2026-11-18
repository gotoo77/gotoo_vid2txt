# Sécurité

La candidate alpha n'est pas un service multiutilisateur sécurisé. Elle exécute des programmes externes qui analysent des fichiers multimédia potentiellement non fiables.

Ne pas la lancer avec des privilèges élevés ou comme outil agentique sans politique d'accès explicite. Pour signaler un problème sensible, préférer un canal privé avec le mainteneur plutôt qu'une issue contenant des données privées.

À qualifier avant release stable : limites de temps/tailles, processus enfants, journaux, chemins concurrents, tests multi-OS et surfaces réseau.
