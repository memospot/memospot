# Para itens de menu, as teclas de atalho do teclado são definidas com "&" em qualquer posição de uma palavra.
# Certifique-se de que nenhuma tecla de atalho conflita com outras no mesmo contexto.

# Menu items
appmenu = &Aplicação
appmenu-quit = &Sair
appmenu-browse-data-directory = &Explorar diretório de dados…
appmenu-settings = &Configurações
appmenu-check-for-updates = &Verificar atualizações…

viewmenu = &Exibição
viewmenu-developer-tools = &Ferramentas do desenvolvedor…
viewmenu-hide-menu-bar = &Esconder barra de menu
viewmenu-refresh = &Atualizar
viewmenu-reload-view = &Recarregar

windowmenu = Jane&la

helpmenu = A&juda
helpmenu-memos-version = &Versão do Memos
helpmenu-memospot-version = &Versão do Memospot
helpmenu-documentation = &Documentação
helpmenu-release-notes = &Notas de lancamento
helpmenu-report-issue = &Reportar um problema…

# Dialogs
dialog-update-title = Atualização disponível
dialog-update-no-update = Nenhuma atualização disponível.
dialog-update-message =
    A versão { $version } está disponível para download.

    Deseja baixá-la?
panic-failed-to-spawn-memos =
    Não foi possível iniciar o Memos:
    {$error}
panic-failed-to-create-data-directory =
    Não foi possível criar o diretório de dados!
    {$dir}
    {$error}
panic-data-directory-is-not-writable =
    O diretório de dados não é gravável!
    {$dir}
panic-unable-to-resolve-custom-data-directory =
    Falha ao resolver o diretório de dados do Memos!
    {$dir}

    Certifique-se de que o caminho exista como um diretório,
    ou remova o ajuste `memos.data` para usar o diretório padrão.
panic-unable-to-create-backup-directory =
    Não foi possível criar o diretório de backup!
    {$dir}
    {$error}
panic-backup-directory-is-a-file =
    O diretório de backup existe como um arquivo!
    {$dir}
panic-backup-directory-is-not-writable =
    O diretório de backup não é gravável!
    {$dir}
panic-database-file-is-not-writable =
    O arquivo de banco de dados não é gravável!
    {$file}
panic-failed-to-connect-to-database =
    Falha ao conectar ao banco de dados:
    {$error}
panic-failed-to-run-database-migrations =
    Falha ao executar migrações do banco de dados:
    {$error}
panic-failed-to-close-database-connection =
    Falha ao fechar a conexão com o banco de dados:
    {$error}
prompt-install-webview-title = Erro no WebView
prompt-install-webview-message =
    O WebView é *obrigatório* para o funcionamento deste
    aplicativo e não está disponível no seu sistema!

    Deseja instalá-lo?
error-failed-to-install-webview =
    Falha ao instalar o WebView:
    {$error}

    Por favor, instale-o manualmente.
