# For menu items, keyboard hot-key mnemonics are set with "&" at any position of a word.
# Make sure they don't conflict with other mnemonics in the same context.

# Menu items
appmenu = アプリケーション(&A)
appmenu-browse-data-directory = &Browse data directory…
appmenu-check-for-updates = &Check for updates…
appmenu-open-in-browser = &Open in browser…
appmenu-quit = 終了(&Q)
appmenu-settings = 設定(&S)
viewmenu = 表示(&V)
viewmenu-developer-tools = 開発ツール(&O)
viewmenu-hide-menu-bar = メニューバーを非表示(&H)
viewmenu-refresh = リフレッシュ(&R)
viewmenu-reload-view = Re&load
windowmenu = ウィンドウ(&W)
helpmenu = ヘルプ (&H)
helpmenu-memos-version = メモのバージョン
helpmenu-memospot-version = メモポットのバージョン(&M)
helpmenu-documentation = ドキュメント(&D)
helpmenu-release-notes = リリースノート(&L)
helpmenu-report-issue = &Report an Issue…
# Dialogs
dialog-update-title = アップデートがあります
dialog-update-no-update = 利用可能な更新はありません。
dialog-update-message =
    バージョン { $version } がダウンロード可能です。
    
    ダウンロードしますか？
panic-failed-to-spawn-memos = メモの生成に失敗しました
panic-failed-to-create-data-directory =
    データディレクトリの作成に失敗しました！
    { $dir }
panic-data-directory-is-not-writable =
    データディレクトリが書き込み可能ではありません!
    { $dir }
panic-unable-to-resolve-custom-data-directory =
    カスタムメモスデータディレクトリの解決に失敗しました！
    { $dir }
    
    パスがディレクトリとして存在することを確認するか、
    設定の `memos を削除します。 ata` はデフォルトのデータパスを使用します。
panic-unable-to-create-backup-directory =
    バックアップディレクトリを作成できません！
    { $dir }
panic-backup-directory-is-a-file =
    バックアップディレクトリはファイルとして存在します!
    { $dir }
panic-backup-directory-is-not-writable =
    バックアップディレクトリが書き込み可能ではありません！
    { $dir }
panic-database-file-is-not-writable =
    データベースファイルが書き込み可能ではありません!
    { $file }
panic-failed-to-connect-to-database = データベースに接続できませんでした
panic-failed-to-run-database-migrations =
    データベース移行の実行に失敗しました:
    { $error }
panic-failed-to-close-database-connection = データベース接続の終了に失敗しました
warn-failed-to-backup-database =
    データベースのバックアップに失敗しました:
    
    { $error }
prompt-install-webview-title = WebView エラー
prompt-install-webview-message =
    このアプリケーションを
    動作させるために WebView が必要です。このシステムでは利用できません！
    
    インストールしますか？
error-failed-to-install-webview =
    WebViewのインストールに失敗しました:
    
    { $error }
    
    手動でインストールしてください。
panic-config-unable-to-create = 設定ファイルを作成できません
panic-config-is-not-a-file =
    指定された設定パスはディレクトリです。
    ファイルでなければなりません！
    { $path }
panic-config-is-not-writable =
    設定ファイルが書き込み可能ではありません!
    { $file }
prompt-config-error-title = 設定エラー
prompt-config-error-message =
    設定ファイルの解析に失敗しました:
    
    { $error }
    
    設定ファイルをリセットしますか?
    (バックアップが作成されます。
panic-config-error =
    Please, fix the configuration file
    manually and restart the application.
panic-config-unable-to-backup = 現在の設定ファイルのバックアップに失敗しました
panic-config-unable-to-reset = 設定ファイルをリセットできません
panic-config-parse-error = 設定ファイルの解析中に致命的なエラーが発生しました
error-config-write-error =
    設定ファイルの書き込みに失敗しました:
    
    { $error }
panic-portpicker-error = メモをバインドするフリーポートが見つかりませんでした！
error-invalid-server-url =
    無効なリモート サーバー URL:
    { $url }
    
    URL は "http" で始まる必要があります。
    設定を確認してください。
panic-unable-to-find-memos-binary = メモサーバーのバイナリが見つかりません！
panic-log-config-write-error =
    ログ設定ファイルの書き込みに失敗しました:
    { $file }
panic-log-config-reset-error =
    ログ設定ファイルのリセットに失敗しました:
    { $file }
    削除してアプリケーションを再起動してください。
