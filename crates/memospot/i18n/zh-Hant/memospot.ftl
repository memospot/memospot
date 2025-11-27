# For menu items, keyboard hot-key mnemonics are set with "&" at any position of a word.
# Make sure they don't conflict with other mnemonics in the same context.

# Menu items
appmenu = &應用程式
appmenu-browse-data-directory = &瀏覽數據目錄…
appmenu-check-for-updates = &檢查更新…
appmenu-open-in-browser = &在瀏覽器中打開…
appmenu-quit = &退出
appmenu-settings = &設置
viewmenu = &查看
viewmenu-developer-tools = &開發者工具
viewmenu-hide-menu-bar = &隱藏菜單欄
viewmenu-new-window = 新視窗
viewmenu-refresh = &刷新
viewmenu-reload-view = 重新&加載
windowmenu = &窗口
helpmenu = &幫助
helpmenu-memos-version = 備忘錄版本
helpmenu-memospot-version = &Memospot版本
helpmenu-documentation = &文檔
helpmenu-release-notes = &發佈說明
helpmenu-report-issue = &報告問題…
# Dialogs
dialog-update-title = 有可用的更新
dialog-update-no-update = 沒有可用的更新。
dialog-update-message = 版本 { $version } 可供下載。您想下載它嗎？
panic-failed-to-spawn-memos = 無法啟動備忘錄
panic-failed-to-create-data-directory = 無法創建數據目錄！ { $dir }
panic-data-directory-is-not-writable = 數據目錄不可寫！ { $dir }
panic-unable-to-resolve-custom-data-directory = 無法解析自定義備忘錄數據目錄！ { $dir } 確保該路徑存在為目錄，或刪除設置 `memos.data` 以使用默認數據路徑。
panic-unable-to-create-backup-directory = 無法創建備份目錄！ { $dir }
panic-backup-directory-is-a-file = 備份目錄存在為文件！ { $dir }
panic-backup-directory-is-not-writable = 備份目錄不可寫！ { $dir }
panic-database-file-is-not-writable = 數據庫文件不可寫！ { $file }
panic-failed-to-connect-to-database = 無法連接到數據庫
panic-failed-to-run-database-migrations = 無法運行數據庫遷移：{ $error }
panic-failed-to-close-database-connection = 無法關閉數據庫連接
warn-failed-to-backup-database = 無法備份數據庫：{ $error }
prompt-install-webview-title = WebView 錯誤
prompt-install-webview-message = 此應用程式需要 *WebView* 才能運行，但在此系統上不可用！您想安裝它嗎？
error-failed-to-install-webview = 無法安裝 WebView：{ $error } 請手動安裝。
panic-config-unable-to-create = 無法創建配置文件
panic-config-is-not-a-file = 提供的配置路徑是目錄。它必須是文件！ { $path }
panic-config-is-not-writable = 配置文件不可寫！ { $file }
prompt-config-error-title = 配置錯誤
prompt-config-error-message = 無法解析配置文件：{ $error } 是否重置配置文件？（將創建備份。）
panic-config-error = 請手動修復配置文件並重新啟動應用程式。
panic-config-unable-to-backup = 無法備份當前配置文件
panic-config-unable-to-reset = 無法重置配置文件
panic-config-parse-error = 解析配置文件時發生致命錯誤
error-config-write-error = 無法寫入配置文件：{ $error }
panic-portpicker-error = 無法找到可用的端口來綁定備忘錄！
error-invalid-server-url = 無效的遠程服務器 URL：{ $url } URL 必須以 "http" 開頭。檢查設置。
panic-unable-to-find-memos-binary = 無法找到備忘錄服務器二進制文件！
panic-log-config-write-error = 無法寫入日誌配置文件：{ $file }
panic-log-config-reset-error = 無法重置日誌配置文件：{ $file } 請刪除它並重新啟動應用程式。
