# For menu items, keyboard hot-key mnemonics are set with "&" at any position of a word.
# Make sure they don't conflict with other mnemonics in the same context.

# Menu items
appmenu = &应用程序
appmenu-browse-data-directory = 浏览数据目录…
appmenu-check-for-updates = 检查更新…
appmenu-open-in-browser = 在浏览器中打开…
appmenu-quit = &退出
appmenu-settings = &设置
viewmenu = &查看
viewmenu-developer-tools = 开发者工具
viewmenu-hide-menu-bar = 隐藏菜单栏
viewmenu-new-window = 新建窗口
viewmenu-refresh = &刷新
viewmenu-reload-view = Re&load
windowmenu = 窗口
helpmenu = 帮助
helpmenu-memos-version = Memos 版本
helpmenu-memospot-version = &Memospot 版本
helpmenu-documentation = 文档
helpmenu-release-notes = &发布说明
helpmenu-report-issue = &报告问题…
# Dialogs
dialog-update-title = 可用更新
dialog-update-no-update = 无可用更新。
dialog-update-message =
    版本 { $version } 可供下载。
    
    您想要下载吗？
dialog-update-failed-title = 自动更新失败
dialog-update-manually-prompt =
    自动更新到版本 { $version }失败：
    
    { $error }
    
    您想手动更新吗？
panic-failed-to-spawn-memos = 啟動 Memos 失敗
panic-failed-to-create-data-directory =
    创建数据目录失败！
    { $dir }
panic-data-directory-is-not-writable =
    数据目录不可写!
    { $dir }
panic-unable-to-resolve-custom-data-directory =
    无法解析自定义Memos 数据目录！
    { $dir }
    
    确保路径作为目录存在，或删除
    设置 `memos'。 使用默认数据路径的 ata` 。
panic-unable-to-create-backup-directory =
    无法创建备份目录！
    { $dir }
panic-backup-directory-is-a-file =
    备份目录作为文件存在！
    { $dir }
panic-backup-directory-is-not-writable =
    备份目录不可写!
    { $dir }
panic-database-file-is-not-writable =
    数据库文件不可写！
    { $file }
panic-failed-to-connect-to-database = 无法连接到数据库
panic-failed-to-run-database-migrations =
    无法运行数据库迁移:
    { $error }
panic-failed-to-close-database-connection = 关闭数据库连接失败
warn-failed-to-backup-database =
    备份数据库失败：
    
    { $error }
prompt-install-webview-title = WebView 错误
prompt-install-webview-message =
    此应用程序*需要* WebView 才能运行，
    而该系统上不可用！
    
    您是否希望安装它？
error-failed-to-install-webview =
    无法安装 WebView:
    
    { $error }
    
    请手动安装它。
panic-config-unable-to-create = 无法创建配置文件
panic-config-is-not-a-file =
    提供的配置路径是一个目录。
    它必须是一个文件！
    { $path }
panic-config-is-not-writable =
    配置文件不可写！
    { $file }
prompt-config-error-title = 配置错误
prompt-config-error-message =
    解析配置文件失败：
    
    { $error }
    
    重置配置文件？
    (将创建备份)
panic-config-error =
    请手动修复配置文件
    并重启应用程序。
panic-config-unable-to-backup = 备份当前配置文件失败
panic-config-unable-to-reset = 无法重置配置文件
panic-config-parse-error = 解析配置文件时发生严重错误
error-config-write-error =
    写入配置文件失败：
    
    { $error }
panic-portpicker-error = 找不到可绑定Memos的自由端口！
error-invalid-server-url =
    无效的远程服务器 URL：
    { $url }
    
    URL必须以“http”开头。
    请检查设置。
panic-unable-to-find-memos-binary = 找不到 Memos 服务器二进制！
panic-log-config-write-error =
    写入日志配置文件失败：
    { $file }
panic-log-config-reset-error =
    重置日志配置文件失败：
    { $file }
    请删除它并重启应用程序。
