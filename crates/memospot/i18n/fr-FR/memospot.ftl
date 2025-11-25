# For menu items, keyboard hot-key mnemonics are set with "&" at any position of a word.
# Make sure they don't conflict with other mnemonics in the same context.

# Menu items
appmenu = &Application
appmenu-browse-data-directory = &Parcourir le répertoire de données…
appmenu-check-for-updates = &Vérifier les mises à jour…
appmenu-open-in-browser = &Ouvrir dans le navigateur…
appmenu-quit = &Quitter
appmenu-settings = &Paramètres
viewmenu = &Voir
viewmenu-developer-tools = Outils de développement
viewmenu-hide-menu-bar = Cacher la barre de menu
viewmenu-refresh = Rafraîchir
viewmenu-reload-view = Re&load
windowmenu = &Fenêtre
helpmenu = &Aide
helpmenu-memos-version = Version des mémos
helpmenu-memospot-version = Version &Memospot
helpmenu-documentation = &Documentation
helpmenu-release-notes = &Notes de version
helpmenu-report-issue = &Signaler un problème…
# Dialogs
dialog-update-title = Mise à jour disponible
dialog-update-no-update = Aucune mise à jour disponible.
dialog-update-message =
    La version { $version } est disponible en téléchargement.
    
    Voulez-vous la télécharger ?
panic-failed-to-spawn-memos = Échec de l'apparition des mémos
panic-failed-to-create-data-directory =
    Impossible de créer le répertoire des données!
    { $dir }
panic-data-directory-is-not-writable =
    Le répertoire des données n'est pas accessible en écriture !
    { $dir }
panic-unable-to-resolve-custom-data-directory =
    Échec de la résolution du répertoire de données des mémos personnalisés!
    { $dir }
    
    Assurez-vous que le chemin existe en tant que répertoire, ou supprimez le paramètre
    `memos. ata` pour utiliser le chemin de données par défaut.
panic-unable-to-create-backup-directory =
    Impossible de créer le répertoire de sauvegarde!
    { $dir }
panic-backup-directory-is-a-file =
    Le répertoire de sauvegarde existe en tant que fichier !
    { $dir }
panic-backup-directory-is-not-writable =
    Le répertoire de sauvegarde n'est pas accessible en écriture !
    { $dir }
panic-database-file-is-not-writable =
    Le fichier de la base de données n'est pas accessible en écriture !
    { $file }
panic-failed-to-connect-to-database = Impossible de se connecter à la base de données
panic-failed-to-run-database-migrations =
    Impossible d'exécuter les migrations de la base de données :
    { $error }
panic-failed-to-close-database-connection = Impossible de fermer la connexion à la base de données
warn-failed-to-backup-database =
    Échec de la sauvegarde de la base de données :
    
    { $error }
prompt-install-webview-title = Erreur WebView
prompt-install-webview-message =
    Une WebView est *nécessaire* pour que cette application fonctionne sur
    et n'est pas disponible sur ce système !
    
    Voulez-vous l'installer ?
error-failed-to-install-webview =
    Impossible d'installer WebView :
    
    { $error }
    
    Veuillez l'installer manuellement.
panic-config-unable-to-create = Impossible de créer le fichier de configuration
panic-config-is-not-a-file =
    Le chemin de configuration fourni est un répertoire.
    Ce doit être un fichier !
    { $path }
panic-config-is-not-writable =
    Le fichier de configuration n'est pas accessible en écriture !
    { $file }
prompt-config-error-title = Erreur de configuration
prompt-config-error-message =
    Impossible d'analyser le fichier de configuration :
    
    { $error }
    
    Réinitialiser le fichier de configuration ?
    (Une sauvegarde sera créée.)
panic-config-error =
    Veuillez corriger le fichier de configuration
    manuellement et redémarrer l'application.
panic-config-unable-to-backup = Échec de la sauvegarde du fichier de configuration actuel
panic-config-unable-to-reset = Impossible de réinitialiser le fichier de configuration
panic-config-parse-error = Erreur fatale lors de l'analyse du fichier de configuration
error-config-write-error =
    Impossible d'écrire le fichier de configuration :
    
    { $error }
panic-portpicker-error = Impossible de trouver un port libre auquel lier les mémos !
error-invalid-server-url =
    URL de serveur distant invalide :
    { $url }
    
    URL doit commencer par "http".
    Vérifiez les paramètres.
panic-unable-to-find-memos-binary = Impossible de trouver le binaire du serveur Memos !
panic-log-config-write-error =
    Impossible d'écrire le fichier de configuration du journal :
    { $file }
panic-log-config-reset-error =
    Impossible de réinitialiser le fichier de configuration du journal :
    { $file }
    Veuillez le supprimer et redémarrer l'application.
