# For menu items, keyboard hot-key mnemonics are set with "&" at any position of a word.
# Make sure they don't conflict with other mnemonics in the same context.

# Menu items
appmenu = &Програма
appmenu-browse-data-directory = &Переглянути каталог даних…
appmenu-check-for-updates = &Check for updates…
appmenu-open-in-browser = &Відкрити у браузері…
appmenu-quit = &Вихід
appmenu-settings = &Налаштування
viewmenu = &Вигляд
viewmenu-developer-tools = &Інструменти розробника
viewmenu-hide-menu-bar = &Приховувати панель меню
viewmenu-refresh = &Оновити
viewmenu-reload-view = Re&load
windowmenu = &Вікно
helpmenu = &Допомога
helpmenu-memos-version = Версія Memos
helpmenu-memospot-version = &Версія Memospot
helpmenu-documentation = &Документація
helpmenu-release-notes = &Історія змін
helpmenu-report-issue = &Повідомити про проблему…
# Dialogs
dialog-update-title = Доступне оновлення
dialog-update-no-update = Оновлення відсутні.
dialog-update-message =
    Version { $version } is available for download.
    
    Do you want to download it?
panic-failed-to-spawn-memos = Не вдалося створити пам'яті
panic-failed-to-create-data-directory =
    Не вдалося створити каталог даних!
    { $dir }
panic-data-directory-is-not-writable =
    Data directory is not writable!
    { $dir }
panic-unable-to-resolve-custom-data-directory =
    Failed to resolve custom Memos data directory!
    { $dir }
    
    Ensure the path exists as a directory, or remove the
    setting `memos.data` to use the default data path.
panic-unable-to-create-backup-directory =
    Unable to create backup directory!
    { $dir }
panic-backup-directory-is-a-file =
    Каталог для резервних копій існує як файл!
    { $dir }
panic-backup-directory-is-not-writable =
    Backup directory is not writable!
    { $dir }
panic-database-file-is-not-writable =
    Database file is not writable!
    { $file }
panic-failed-to-connect-to-database = Не вдалося підключитися до бази даних
panic-failed-to-run-database-migrations =
    Не вдалося запустити міграцію бази даних:
    { $error }
panic-failed-to-close-database-connection = Не вдалося закрити підключення до бази даних
warn-failed-to-backup-database =
    Не вдалося створити резервну копію:
    
    { $error }
prompt-install-webview-title = Помилка WebView
prompt-install-webview-message =
    A WebView is *required* for this application to
    work and it is not available on this system!
    
    Do you want to install it?
error-failed-to-install-webview =
    Не вдалося встановити WebView:
    
    { $error }
    
    Будь ласка, встановіть його вручну.
panic-config-unable-to-create = Не вдалося створити файл конфігурації
panic-config-is-not-a-file =
    Provided configuration path is a directory.
    It must be a file!
    { $path }
panic-config-is-not-writable =
    Configuration file is not writable!
    { $file }
prompt-config-error-title = Помилка конфігурації
prompt-config-error-message =
    Не вдалося обробити файл конфігурації:
    
    { $error }
    
    Скинути файл конфігурації?
    (Резервну копію буде створено.)
panic-config-error =
    Будь ласка, виправте файл конфігурації
    вручну і перезапустіть додаток.
panic-config-unable-to-backup = Не вдалося зарезервувати поточний файл конфігурації
panic-config-unable-to-reset = Неможливо скинути файл конфігурації
panic-config-parse-error = Критична помилка під час аналізу файлу конфігурації
error-config-write-error =
    Failed to write configuration file:
    
    { $error }
panic-portpicker-error = Не вдалося знайти вільний порт для прив'язки приміток!
error-invalid-server-url =
    Неправильний URL віддаленого сервера:
    { $url }
    
    URL повинен починатися з "http".
    Перевірте налаштування.
panic-unable-to-find-memos-binary = Не вдалося знайти файл Memos сервера!
panic-log-config-write-error =
    Failed to write log configuration file:
    { $file }
panic-log-config-reset-error =
    Failed to reset the log configuration file:
    { $file }
    Please delete it and restart the application.
