# For menu items, keyboard hot-key mnemonics are set with "&" at any position of a word.
# Make sure they don't conflict with other mnemonics in the same context.

# Menu items
appmenu = &Приложение
appmenu-browse-data-directory = &Просмотр каталога данных…
appmenu-check-for-updates = &Проверять наличие обновлений…
appmenu-open-in-browser = &Открыть в браузере…
appmenu-quit = &Выйти
appmenu-settings = &Настройки
viewmenu = Вид
viewmenu-developer-tools = &Инструменты разработчика
viewmenu-hide-menu-bar = &Скрыть панель меню
viewmenu-refresh = &Обновить
viewmenu-reload-view = Re&load
windowmenu = &Окно
helpmenu = &Помощь
helpmenu-memos-version = Версия примечаний
helpmenu-memospot-version = &Версия Memospot
helpmenu-documentation = &Документация
helpmenu-release-notes = &Заметки о выпуске
helpmenu-report-issue = &Сообщить о проблеме…
# Dialogs
dialog-update-title = Доступно обновление
dialog-update-no-update = Нет доступных обновлений.
dialog-update-message =
    Версия { $version } доступна для скачивания.
    
    Вы хотите скачать его?
panic-failed-to-spawn-memos = Не удалось создать примечания
panic-failed-to-create-data-directory =
    Не удалось создать каталог данных!
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
    Невозможно создать резервную копию!
    { $dir }
panic-backup-directory-is-a-file =
    Backup directory exists as a file!
    { $dir }
panic-backup-directory-is-not-writable =
    Backup directory is not writable!
    { $dir }
panic-database-file-is-not-writable =
    Database file is not writable!
    { $file }
panic-failed-to-connect-to-database = Не удалось подключиться к базе данных
panic-failed-to-run-database-migrations =
    Не удалось выполнить миграцию базы данных:
    { $error }
panic-failed-to-close-database-connection = Не удалось закрыть соединение с базой данных
warn-failed-to-backup-database =
    Ошибка резервного копирования базы данных:
    
    { $error }
prompt-install-webview-title = Ошибка WebView
prompt-install-webview-message =
    A WebView is *required* for this application to
    work and it is not available on this system!
    
    Do you want to install it?
error-failed-to-install-webview =
    Не удалось установить WebView:
    
    { $error }
    
    Пожалуйста, установите его вручную.
panic-config-unable-to-create = Не удается создать файл конфигурации
panic-config-is-not-a-file =
    Provided configuration path is a directory.
    It must be a file!
    { $path }
panic-config-is-not-writable =
    Configuration file is not writable!
    { $file }
prompt-config-error-title = Ошибка конфигурации
prompt-config-error-message =
    Не удалось разобрать конфигурационный файл:
    
    { $error }
    
    Сбросить конфигурационный файл?
    (будет создана резервная копия)
panic-config-error =
    Пожалуйста, исправьте конфигурационный файл
    вручную и перезапустите приложение.
panic-config-unable-to-backup = Не удалось сделать резервную копию текущего файла конфигурации
panic-config-unable-to-reset = Не удалось сбросить файл конфигурации
panic-config-parse-error = Фатальная ошибка при разборе конфигурационного файла
error-config-write-error =
    Не удалось записать файл конфигурации:
    
    { $error }
panic-portpicker-error = Не удалось найти свободный порт для привязки Memos!
error-invalid-server-url =
    Неверный URL удаленного сервера:
    { $url }
    
    URL должен начинаться с "http".
    Проверьте настройки.
panic-unable-to-find-memos-binary = Не удалось найти бинарный файл Memos сервера!
panic-log-config-write-error =
    Не удалось записать файл конфигурации журнала:
    { $file }
panic-log-config-reset-error =
    Не удалось сбросить конфигурационный файл журнала:
    { $file }
    Удалите его и перезапустите приложение.
