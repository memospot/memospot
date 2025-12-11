# For menu items, keyboard hot-key mnemonics are set with "&" at any position of a word.
# Make sure they don't conflict with other mnemonics in the same context.

# Menu items
appmenu = &Aplicação
appmenu-browse-data-directory = &Explorar diretório de dados…
appmenu-check-for-updates = &Verificar atualizações…
appmenu-open-in-browser = Abrir no &navegador…
appmenu-quit = &Sair
appmenu-settings = &Configurações
viewmenu = &Exibição
viewmenu-developer-tools = &Ferramentas do desenvolvedor…
viewmenu-hide-menu-bar = &Esconder barra de menu
viewmenu-new-window = &Nova Janela
viewmenu-refresh = &Atualizar
viewmenu-reload-view = &Recarregar
windowmenu = Jane&la
helpmenu = A&juda
helpmenu-memos-version = &Versão do Memos
helpmenu-memospot-version = &Versão do Memospot
helpmenu-documentation = &Documentação
helpmenu-release-notes = &Notas de lançamento
helpmenu-report-issue = &Reportar um problema…
# Dialogs
dialog-update-title = Atualização disponível
dialog-update-no-update = Nenhuma atualização disponível.
dialog-update-message =
    A versão { $version } está disponível para download.
    
    Deseja baixá-la?
dialog-update-failed-title = Auto-atualização falhou
dialog-update-manually-prompt =
    Falha ao atualizar automaticamente para a versão { $version }:
    
    { $error }
    
    Gostaria de atualizar manualmente?
panic-failed-to-spawn-memos = Não foi possível iniciar o Memos
panic-failed-to-create-data-directory =
    Não foi possível criar o diretório de dados!
    { $dir }
panic-data-directory-is-not-writable =
    O diretório de dados não é gravável!
    { $dir }
panic-unable-to-resolve-custom-data-directory =
    Falha ao resolver o diretório de dados do Memos!
    { $dir }
    
    Certifique-se de que o caminho exista como um diretório,
    ou remova o ajuste `memos.data` para usar o diretório padrão.
panic-unable-to-create-backup-directory =
    Não foi possível criar o diretório de backup!
    { $dir }
panic-backup-directory-is-a-file =
    O diretório de backup existe como um arquivo!
    { $dir }
panic-backup-directory-is-not-writable =
    O diretório de backup não é gravável!
    { $dir }
panic-database-file-is-not-writable =
    O arquivo de banco de dados não é gravável!
    { $file }
panic-failed-to-connect-to-database = Falha ao conectar ao banco de dados
panic-failed-to-run-database-migrations =
    Falha ao executar migrações do banco de dados:
    { $error }
panic-failed-to-close-database-connection = Falha ao fechar a conexão com o banco de dados
warn-failed-to-backup-database =
    Falha ao fazer backup do banco de dados:
    
    { $error }
prompt-install-webview-title = Erro no WebView
prompt-install-webview-message =
    O WebView é *obrigatório* para o funcionamento deste
    aplicativo e não está disponível no seu sistema!
    
    Deseja instalá-lo?
error-failed-to-install-webview =
    Falha ao instalar o WebView:
    
    { $error }
    
    Por favor, instale-o manualmente.
panic-config-unable-to-create = Não foi possível criar o arquivo de configuração
panic-config-is-not-a-file =
    O caminho fornecido para a configuração é um diretório.
    Deve ser um arquivo!
    { $path }
panic-config-is-not-writable =
    O arquivo de configuração não é gravável!
    { $file }
prompt-config-error-title = Erro na configuração
prompt-config-error-message =
    Erro ao ler o arquivo de configuração:
    
    { $error }
    
    Redefinir o arquivo de configuração?
    (Será criado um backup.)
panic-config-error =
    Por favor, corrija o arquivo de configuração
    manualmente e reinicie o aplicativo.
panic-config-unable-to-backup = Falha ao fazer backup do arquivo de configuração atual
panic-config-unable-to-reset = Não foi possível redefinir o arquivo de configuração
panic-config-parse-error = Erro fatal ao ler o arquivo de configuração
error-config-write-error =
    Falha ao gravar o arquivo de configuração:
    
    { $error }
panic-portpicker-error = Falha ao encontrar uma porta livre para o Memos escutar!
error-invalid-server-url =
    URL do servidor inválida:
    { $url }
    
    A URL deve iniciar com "http".
    Verifique as configurações.
panic-unable-to-find-memos-binary = Não foi possível encontrar o binário do servidor do Memos!
panic-log-config-write-error =
    Falha ao gravar o arquivo de configuração de log:
    { $file }
panic-log-config-reset-error =
    Falha ao redefinir o arquivo de configuração de log:
    { $file }
    Por favor, exclua este arquivo e reinicie o aplicativo.
