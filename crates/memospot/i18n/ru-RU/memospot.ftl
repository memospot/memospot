# For menu items, keyboard hot-key mnemonics are set with "&" at any position of a word.
# Make sure they don't conflict with other mnemonics in the same context.

# Menu items
appmenu = &Приложение
appmenu-browse-data-directory = &Просмотреть каталог данных…
appmenu-check-for-updates = &Проверить наличие обновлений…
appmenu-open-in-browser = &Открыть в браузере…
appmenu-quit = &Выйти
appmenu-settings = &Настройки
viewmenu = &Просмотр
viewmenu-developer-tools = &Инструменты разработчика
viewmenu-hide-menu-bar = &Скрыть панель меню
viewmenu-new-window = &Новое окно
viewmenu-refresh = &Обновить
viewmenu-reload-view = Перезагрузить
windowmenu = &Окно
helpmenu = &Помощь
helpmenu-memos-version = Версия Мемо
helpmenu-memospot-version = &Версия Memospot
helpmenu-documentation = &Документация
helpmenu-release-notes = &Примечания к выпуску
helpmenu-report-issue = &Сообщить об ошибке…
# Dialogs
dialog-update-title = Доступно обновление
dialog-update-no-update = Обновление недоступно.
dialog-update-message = Версия { $version } доступна для загрузки. Вы хотите загрузить её?
dialog-update-failed-title = Ошибка автообновления
dialog-update-manually-prompt =
    Ошибка автоматического обновления до версии { $version }:
    
    { $error }
    
    Вы хотите обновить вручную?
panic-failed-to-spawn-memos = Не удалось запустить Мемо
panic-failed-to-create-data-directory = Не удалось создать каталог данных! { $dir }
panic-data-directory-is-not-writable = Каталог данных недоступен для записи! { $dir }
panic-unable-to-resolve-custom-data-directory = Не удалось разрешить пользовательский каталог данных Мемо! { $dir } Убедитесь, что путь существует как каталог, или удалите настройку `memos.data`, чтобы использовать путь по умолчанию.
panic-unable-to-create-backup-directory = Не удалось создать каталог резервной копии! { $dir }
panic-backup-directory-is-a-file = Каталог резервной копии существует как файл! { $dir }
panic-backup-directory-is-not-writable = Каталог резервной копии недоступен для записи! { $dir }
panic-database-file-is-not-writable = Файл базы данных недоступен для записи! { $file }
panic-failed-to-connect-to-database = Не удалось подключиться к базе данных
panic-failed-to-run-database-migrations = Не удалось выполнить миграции базы данных: { $error }
panic-failed-to-close-database-connection = Не удалось закрыть соединение с базой данных
warn-failed-to-backup-database = Не удалось создать резервную копию базы данных: { $error }
prompt-install-webview-title = Ошибка WebView
prompt-install-webview-message = WebView *обязателен* для работы этого приложения, и он недоступен в этой системе! Вы хотите установить его?
error-failed-to-install-webview = Не удалось установить WebView: { $error } Пожалуйста, установите его вручную.
panic-config-unable-to-create = Не удалось создать файл конфигурации
panic-config-is-not-a-file = Предоставленный путь конфигурации является каталогом. Это должен быть файл! { $path }
panic-config-is-not-writable = Файл конфигурации недоступен для записи! { $file }
prompt-config-error-title = Ошибка конфигурации
prompt-config-error-message = Не удалось разобрать файл конфигурации: { $error } Сбросить файл конфигурации? (Будет создана резервная копия.)
panic-config-error = Пожалуйста, исправьте файл конфигурации вручную и перезапустите приложение.
panic-config-unable-to-backup = Не удалось создать резервную копию текущего файла конфигурации
panic-config-unable-to-reset = Не удалось сбросить файл конфигурации
panic-config-parse-error = Фатальная ошибка при разборе файла конфигурации
error-config-write-error = Не удалось записать файл конфигурации: { $error }
panic-portpicker-error = Не удалось найти свободный порт для привязки Мемо!
error-invalid-server-url = Недопустимый URL удаленного сервера: { $url } URL должен начинаться с "http". Проверьте настройки.
panic-unable-to-find-memos-binary = Не удалось найти бинарный файл сервера Мемо!
panic-log-config-write-error = Не удалось записать файл конфигурации журнала: { $file }
panic-log-config-reset-error = Не удалось сбросить файл конфигурации журнала: { $file } Пожалуйста, удалите его и перезапустите приложение.
