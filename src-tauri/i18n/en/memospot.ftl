# For menu items, keyboard hot-key mnemonics are set with "&" at any position of a word.
# Make sure they don't conflict with other mnemonics in the same context.

# Menu items
appmenu = &App
appmenu-quit = &Quit
appmenu-browse-data-directory = &Browse data directory…
appmenu-settings = &Settings
appmenu-check-for-updates = &Check for updates…

viewmenu = &View
viewmenu-developer-tools = &Developer Tools
viewmenu-hide-menu-bar = &Hide menu bar
viewmenu-refresh = &Refresh
viewmenu-reload-view = Re&load

windowmenu = &Window

helpmenu = &Help
helpmenu-memos-version = Memos version
helpmenu-memospot-version = &Memospot version
helpmenu-documentation = &Documentation
helpmenu-release-notes = &Release Notes
helpmenu-report-issue = &Report an Issue…

# Dialogs
dialog-update-title = Update available
dialog-update-no-update = No update available.
dialog-update-message =
    Version { $version } is available for download.

    Do you want to download it?
panic-failed-to-spawn-memos =
    Failed to spawn Memos:
    {$error}
panic-failed-to-create-data-directory =
    Failed to create data directory!
    {$dir}
    {$error}
panic-data-directory-is-not-writable =
    Data directory is not writable!
    {$dir}
panic-unable-to-resolve-custom-data-directory =
    Failed to resolve custom Memos data directory!
    {$dir}

    Ensure the path exists as a directory, or remove the
    setting `memos.data` to use the default data path.
panic-unable-to-create-backup-directory =
    Unable to create backup directory!
    {$dir}
    {$error}
panic-backup-directory-is-a-file =
    Backup directory exists as a file!
    {$dir}
panic-backup-directory-is-not-writable =
    Backup directory is not writable!
    {$dir}
panic-database-file-is-not-writable =
    Database file is not writable!
    {$file}
panic-failed-to-connect-to-database =
    Failed to connect to the database:
    {$error}
panic-failed-to-run-database-migrations =
    Failed to run database migrations:
    {$error}
panic-failed-to-close-database-connection =
    Failed to close database connection:
    {$error}
prompt-install-webview-title = WebView Error
prompt-install-webview-message =
    A WebView is *required* for this application to
    work and it is not available on this system!

    Do you want to install it?
error-failed-to-install-webview =
    Failed to install WebView:
    {$error}
    Please install it manually.

