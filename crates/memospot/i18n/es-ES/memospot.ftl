# For menu items, keyboard hot-key mnemonics are set with "&" at any position of a word.
# Make sure they don't conflict with other mnemonics in the same context.

# Menu items
appmenu = &Aplicación
appmenu-browse-data-directory = &Navegar directorio de datos…
appmenu-check-for-updates = &Comprobar actualizaciones…
appmenu-open-in-browser = &Abrir en el navegador…
appmenu-quit = &Salir
appmenu-settings = &Ajustes
viewmenu = &Ver
viewmenu-developer-tools = &Herramientas de Desarrollador
viewmenu-hide-menu-bar = &Ocultar barra de menú
viewmenu-refresh = &Actualizar
viewmenu-reload-view = Re&load
windowmenu = &Ventana
helpmenu = &Ayuda
helpmenu-memos-version = Versión de Memos
helpmenu-memospot-version = Versión de &Memospot
helpmenu-documentation = &Documentación
helpmenu-release-notes = &Notas de lanzamiento
helpmenu-report-issue = Informar un problema…
# Dialogs
dialog-update-title = Actualización disponible
dialog-update-no-update = No hay actualizaciones disponibles.
dialog-update-message =
    La versión { $version } está disponible para la descarga.
    
    ¿Desea descargarla?
panic-failed-to-spawn-memos = Error al generar Memos
panic-failed-to-create-data-directory =
    ¡Error al crear el directorio de datos!
    { $dir }
panic-data-directory-is-not-writable =
    ¡No se puede escribir en el directorio de datos!
    { $dir }
panic-unable-to-resolve-custom-data-directory =
    ¡Error al resolver el directorio de datos de Memos personalizado!
    { $dir }
    
    Asegúrese de que la ruta existe como un directorio, o elimine el ajuste
    `memos. ata` para usar la ruta de datos por defecto.
panic-unable-to-create-backup-directory =
    ¡No se puede crear el directorio de copia de seguridad!
    { $dir }
panic-backup-directory-is-a-file =
    ¡Directorio de copia de seguridad existe como archivo!
    { $dir }
panic-backup-directory-is-not-writable =
    ¡No se puede escribir en el directorio de copia de seguridad!
    { $dir }
panic-database-file-is-not-writable =
    ¡El archivo de base de datos no es escribible!
    { $file }
panic-failed-to-connect-to-database = Error al conectar a la base de datos
panic-failed-to-run-database-migrations =
    Error al ejecutar las migraciones de base de datos:
    { $error }
panic-failed-to-close-database-connection = Error al cerrar la conexión de base de datos
warn-failed-to-backup-database =
    Error al respaldar la base de datos:
    
    { $error }
prompt-install-webview-title = Error de WebView
prompt-install-webview-message =
    Se requiere una vista web para que esta aplicación 
    funcione, pero no está disponible en este sistema.
    
    ¿Quieres instalarla?
error-failed-to-install-webview =
    Error al instalar WebView:
    
    { $error }
    
    Instálalo manualmente.
panic-config-unable-to-create = No se puede crear el archivo de configuración
panic-config-is-not-a-file =
    La ruta de configuración proporcionada es un directorio.
    ¡Debe ser un archivo!
    { $path }
panic-config-is-not-writable =
    ¡El archivo de configuración no es escribible!
    { $file }
prompt-config-error-title = Error de configuración
prompt-config-error-message =
    Error al analizar el archivo de configuración:
    
    { $error }
    
    ¿Reiniciar el archivo de configuración?
    (Se creará una copia de seguridad.)
panic-config-error =
    Por favor, corrija el archivo de configuración
    manualmente y reinicie la aplicación.
panic-config-unable-to-backup = Error al respaldar el archivo de configuración actual
panic-config-unable-to-reset = No se puede restablecer el archivo de configuración
panic-config-parse-error = Error fatal al analizar el archivo de configuración
error-config-write-error =
    Error al escribir el archivo de configuración:
    
    { $error }
panic-portpicker-error = ¡Error al encontrar un puerto libre para enlazar Memos !
error-invalid-server-url =
    URL del servidor remoto no válida:
    { $url }
    
    URL debe comenzar con "http".
    Compruebe la configuración.
panic-unable-to-find-memos-binary = ¡No se puede encontrar el binario del servidor Memos!
panic-log-config-write-error =
    Error al escribir archivo de configuración de registro:
    { $file }
panic-log-config-reset-error =
    Error al restablecer el archivo de configuración de registro:
    { $file }
    Por favor, elimínalo y reinicia la aplicación.
