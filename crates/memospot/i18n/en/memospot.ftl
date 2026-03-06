appmenu = &Application
appmenu-browse-data-directory = &Browse data directory…
appmenu-check-for-updates = &Check for updates…
appmenu-open-in-browser = &Open in browser…
appmenu-quit = &Quit
appmenu-settings = &Settings
dialog-generic-error = Error
dialog-generic-info = Information
dialog-update-failed-title = Auto-update failed
dialog-update-manually-prompt = Failed to auto update to version { $version }:

    { $error }

    Would you like to update manually?
dialog-update-message = Version { $version } is available for download.

    Do you want to download it?
dialog-update-no-update = No update available.
dialog-update-title = Update available
error-config-write-error = Failed to write configuration file:

    { $error }
error-failed-to-install-webview = Failed to install WebView:

    { $error }

    Please install it manually.
error-invalid-server-url = Invalid remote server URL:
    { $url }

    URL must start with "http".
    Check the settings.
helpmenu = &Help
helpmenu-documentation = &Documentation
helpmenu-memos-version = Memos version
helpmenu-memospot-version = &Memospot version
helpmenu-release-notes = &Release Notes
helpmenu-report-issue = &Report an Issue…
panic-backup-directory-is-a-file = Backup directory exists as a file!
    { $dir }
panic-backup-directory-is-not-writable = Backup directory is not writable!
    { $dir }
panic-config-error = Please, fix the configuration file
    manually and restart the application.
panic-config-is-not-a-file = Provided configuration path is a directory.
    It must be a file!
    { $path }
panic-config-is-not-writable = Configuration file is not writable!
    { $file }
panic-config-parse-error = Fatal error while parsing the configuration file
panic-config-unable-to-backup = Failed to backup the current configuration file
panic-config-unable-to-create = Unable to create configuration file
panic-config-unable-to-reset = Unable to reset the configuration file
panic-data-directory-is-not-writable = Data directory is not writable!
    { $dir }
panic-database-file-is-not-writable = Database file is not writable!
    { $file }
panic-failed-to-close-database-connection = Failed to close database connection
panic-failed-to-connect-to-database = Failed to connect to the database
panic-failed-to-create-data-directory = Failed to create data directory!
    { $dir }
panic-failed-to-run-database-migrations = Failed to run database migrations:
    { $error }
panic-failed-to-spawn-memos = Failed to spawn Memos
panic-log-config-reset-error = Failed to reset the log configuration file:
    { $file }
    Please delete it and restart the application.
panic-log-config-write-error = Failed to write log configuration file:
    { $file }
panic-portpicker-error = Failed to find a free port to bind Memos to!
panic-unable-to-create-backup-directory = Unable to create backup directory!
    { $dir }
panic-unable-to-find-memos-binary = Unable to find Memos server binary!
panic-unable-to-resolve-custom-data-directory = Failed to resolve custom Memos data directory!
    { $dir }

    Ensure the path exists as a directory, or remove the
    setting `memos.data` to use the default data path.
prompt-config-error-message = Failed to parse configuration file:

    { $error }

    Reset the configuration file?
    (A backup will be created.)
prompt-config-error-title = Configuration Error
prompt-install-webview-message = A WebView is *required* for this application to
    work and it is not available on this system!

    Do you want to install it?
prompt-install-webview-title = WebView Error
viewmenu = &View
viewmenu-developer-tools = &Developer Tools
viewmenu-new-window = &New Window
viewmenu-refresh = &Refresh
viewmenu-reload-view = Re&load
viewmenu-reset-zoom = &Reset Zoom
viewmenu-toggle-menu-bar = T&oggle menu bar
viewmenu-zoom-in = Zoom &In
viewmenu-zoom-out = Zoom &Out
warn-failed-to-backup-database = Failed to backup database:

    { $error }
windowmenu = &Window
