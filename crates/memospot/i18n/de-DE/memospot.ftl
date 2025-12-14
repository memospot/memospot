# For menu items, keyboard hot-key mnemonics are set with "&" at any position of a word.
# Make sure they don't conflict with other mnemonics in the same context.

# Menu items
appmenu = &Anwendung
appmenu-browse-data-directory = Datenverzeichnis… durchsuchen
appmenu-check-for-updates = Suche nach Aktualisierungen…
appmenu-open-in-browser = Im Browser… &öffnen
appmenu-quit = &Beenden
appmenu-settings = &Einstellungen
viewmenu = &Ansicht
viewmenu-developer-tools = &Entwicklerwerkzeuge
viewmenu-hide-menu-bar = &Menüleiste ausblenden
viewmenu-new-window = &Neues Fenster
viewmenu-refresh = &Aktualisieren
viewmenu-reload-view = Re&load
windowmenu = &Fenster
helpmenu = &Hilfe
helpmenu-memos-version = Memos Version
helpmenu-memospot-version = &Memospot-Version
helpmenu-documentation = &Dokumentation
helpmenu-release-notes = &Versionshinweise
helpmenu-report-issue = Ein Problem &melden…
# Dialogs
dialog-generic-error = Fehler
dialog-generic-info = Informationen
dialog-update-title = Update verfügbar
dialog-update-no-update = Kein Update verfügbar.
dialog-update-message =
    Version { $version } steht zum Download zur Verfügung.
    
    Möchten Sie sie herunterladen?
dialog-update-failed-title = Auto-Update fehlgeschlagen
dialog-update-manually-prompt =
    Fehler beim automatischen Update auf Version { $version }:
    
    { $error }
    
    Möchten Sie manuell aktualisieren?
panic-failed-to-spawn-memos = Fehler beim Spawnen von Memos
panic-failed-to-create-data-directory =
    Datenverzeichnis konnte nicht erstellt werden!
    { $dir }
panic-data-directory-is-not-writable =
    Datenverzeichnis ist nicht beschreibbar!
    { $dir }
panic-unable-to-resolve-custom-data-directory =
    Fehler beim Auflösen des benutzerdefinierten Memos-Datenverzeichniss!
    { $dir }
    
    Stellen Sie sicher, dass der Pfad als Verzeichnis existiert, oder entfernen Sie die
    Einstellung `memos. ata` um den Standard-Datenpfad zu verwenden.
panic-unable-to-create-backup-directory =
    Sicherungsverzeichnis kann nicht erstellt werden!
    { $dir }
panic-backup-directory-is-a-file =
    Sicherungsverzeichnis existiert als Datei!
    { $dir }
panic-backup-directory-is-not-writable =
    Backup-Verzeichnis ist nicht beschreibbar!
    { $dir }
panic-database-file-is-not-writable =
    Datenbankdatei ist nicht beschreibbar!
    { $file }
panic-failed-to-connect-to-database = Verbindung zur Datenbank fehlgeschlagen
panic-failed-to-run-database-migrations =
    Fehler beim Ausführen von Datenbankmigrationen:
    { $error }
panic-failed-to-close-database-connection = Datenbankverbindung konnte nicht geschlossen werden
warn-failed-to-backup-database =
    Sicherung der Datenbank fehlgeschlagen:
    
    { $error }
prompt-install-webview-title = WebView-Fehler
prompt-install-webview-message =
    Eine WebView ist *erforderlich* für diese Anwendung zum Arbeiten mit
    und sie ist auf diesem System nicht verfügbar!
    
    Möchten Sie sie installieren?
error-failed-to-install-webview =
    WebView konnte nicht installiert werden:
    
    { $error }
    
    Bitte manuell installieren.
panic-config-unable-to-create = Konfigurationsdatei kann nicht erstellt werden
panic-config-is-not-a-file =
    Der angegebene Konfigurationspfad ist ein Verzeichnis.
    Es muss eine Datei sein!
    { $path }
panic-config-is-not-writable =
    Konfigurationsdatei ist nicht beschreibbar!
    { $file }
prompt-config-error-title = Konfigurationsfehler
prompt-config-error-message =
    Fehler beim Parsen der Konfigurationsdatei:
    
    { $error }
    
    Zurücksetzen der Konfigurationsdatei?
    (Ein Backup wird erstellt.)
panic-config-error =
    Bitte korrigieren Sie die Konfigurationsdatei
    manuell und starten Sie die Anwendung neu.
panic-config-unable-to-backup = Sicherung der aktuellen Konfigurationsdatei fehlgeschlagen
panic-config-unable-to-reset = Kann die Konfigurationsdatei nicht zurücksetzen
panic-config-parse-error = Schwerer Fehler beim Parsen der Konfigurationsdatei
error-config-write-error =
    Fehler beim Schreiben der Konfigurationsdatei:
    
    { $error }
panic-portpicker-error = Konnte keinen freien Port für Memos finden!
error-invalid-server-url =
    Ungültige Remote-Server-URL:
    { $url }
    
    URL muss mit "http" beginnen.
    Überprüfen Sie die Einstellungen.
panic-unable-to-find-memos-binary = Memos Server binär nicht gefunden!
panic-log-config-write-error =
    Fehler beim Schreiben der Protokollkonfigurationsdatei:
    { $file }
panic-log-config-reset-error =
    Fehler beim Zurücksetzen der Protokollkonfigurationsdatei:
    { $file }
    Bitte löschen und die Anwendung neu starten.
