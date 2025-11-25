# For menu items, keyboard hot-key mnemonics are set with "&" at any position of a word.
# Make sure they don't conflict with other mnemonics in the same context.

# Menu items
appmenu = &Aplicação
appmenu-browse-data-directory = &Procurar diretório de dados…
appmenu-check-for-updates = &Verificar atualizações…
appmenu-open-in-browser = &Abrir no navegador…
appmenu-quit = &Encerrar
appmenu-settings = Configurações(&S)
viewmenu = &Visualização
viewmenu-developer-tools = &Ferramentas de Desenvolvedor
viewmenu-hide-menu-bar = &Ocultar barra de menu
viewmenu-refresh = &Atualizar
viewmenu-reload-view = Re&load
windowmenu = &Janela
helpmenu = &Ajuda
helpmenu-memos-version = Versão de demos
helpmenu-memospot-version = Versão do &Memospot
helpmenu-documentation = &Documentação
helpmenu-release-notes = &Notas de Lançamento
helpmenu-report-issue = &Relatar um problema…
# Dialogs
dialog-update-title = Atualização disponível
dialog-update-no-update = Nenhuma atualização disponível.
dialog-update-message =
    Versão { $version } está disponível para download.
    
    Você quer baixá-la?
panic-failed-to-spawn-memos = Falha ao gerar Memos
panic-failed-to-create-data-directory =
    Falha ao criar o diretório de dados!
    { $dir }
panic-data-directory-is-not-writable =
    O diretório de dados não tem permissões de escrita!
    { $dir }
panic-unable-to-resolve-custom-data-directory =
    Failed to resolve custom Memos data directory!
    { $dir }
    
    Ensure the path exists as a directory, or remove the
    setting `memos.data` to use the default data path.
panic-unable-to-create-backup-directory =
    Não foi possível criar o diretório de backup!
    { $dir }
panic-backup-directory-is-a-file =
    Diretório de backup existe como um arquivo!
    { $dir }
panic-backup-directory-is-not-writable =
    Diretório de backup não tem permissões de escrita!
    { $dir }
panic-database-file-is-not-writable =
    O arquivo de banco de dados não é gravável!
    { $file }
panic-failed-to-connect-to-database = Falha ao conectar ao banco de dados
panic-failed-to-run-database-migrations =
    Falha ao executar as migrações do banco de dados:
    { $error }
panic-failed-to-close-database-connection = Falha ao fechar a conexão do banco de dados
warn-failed-to-backup-database =
    Falha ao fazer backup do banco de dados:
    
    { $error }
prompt-install-webview-title = Erro no WebView
prompt-install-webview-message =
    Um WebView é *necessário* para esta aplicação funcionar como
    e não está disponível neste sistema!
    
    Você quer instalá-la?
error-failed-to-install-webview =
    Falha ao instalar WebView:
    
    { $error }
    
    Por favor, instale-o manualmente.
panic-config-unable-to-create = Não foi possível criar o arquivo de configuração
panic-config-is-not-a-file =
    Provided configuration path is a directory.
    It must be a file!
    { $path }
panic-config-is-not-writable =
    O arquivo de configuração não é gravável!
    { $file }
prompt-config-error-title = Erro de configuração
prompt-config-error-message =
    Falha ao analisar o arquivo de configuração:
    
    { $error }
    
    Redefinir o arquivo de configuração?
    (Um backup será criado.)
panic-config-error =
    Por favor, corrija o arquivo de configuração
    manualmente e reinicie o aplicativo.
panic-config-unable-to-backup = Falha ao fazer backup do arquivo de configuração atual
panic-config-unable-to-reset = Não foi possível redefinir o arquivo de configuração
panic-config-parse-error = Erro fatal ao analisar o arquivo de configuração
error-config-write-error =
    Falha ao gravar o arquivo de configuração:
    
    { $error }
panic-portpicker-error = Falha ao encontrar uma porta gratuita para vincular Memos para!
error-invalid-server-url =
    URL de servidor remoto inválida:
    { $url }
    
    URL deve começar com "http".
    Verifique as configurações.
panic-unable-to-find-memos-binary = Não foi possível encontrar o binário do servidor Mdemos!
panic-log-config-write-error =
    Falha ao gravar arquivo de configuração de log:
    { $file }
panic-log-config-reset-error =
    Falha ao redefinir o arquivo de configuração de log:
    { $file }
    Por favor, apague-o e reinicie o aplicativo.
