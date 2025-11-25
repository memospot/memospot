# For menu items, keyboard hot-key mnemonics are set with "&" at any position of a word.
# Make sure they don't conflict with other mnemonics in the same context.

# Menu items
appmenu = &التطبيق
appmenu-browse-data-directory = &تصفح دليل البيانات…
appmenu-check-for-updates = &التحقق من وجود تحديثات…
appmenu-open-in-browser = &فتح في المتصفح…
appmenu-quit = &إنهاء
appmenu-settings = &الإعدادات
viewmenu = &عرض
viewmenu-developer-tools = أدوات &المطور
viewmenu-hide-menu-bar = &إخفاء شريط القائمة
viewmenu-refresh = &تحديث
viewmenu-reload-view = Re&load
windowmenu = &نافذة
helpmenu = &مساعدة
helpmenu-memos-version = إصدار المذكرات
helpmenu-memospot-version = إصدار &Memospot
helpmenu-documentation = &التوثيق
helpmenu-release-notes = &اصدار ملاحظات
helpmenu-report-issue = &التبليغ عن مشكلة…
# Dialogs
dialog-update-title = التحديث متاح
dialog-update-no-update = لا يوجد تحديث متاح.
dialog-update-message =
    الإصدار { $version } متاح للتحميل.
    
    هل تريد تحميله؟
panic-failed-to-spawn-memos = فشل في نشر المذكرات
panic-failed-to-create-data-directory =
    فشل في إنشاء دليل البيانات!
    { $dir }
panic-data-directory-is-not-writable =
    دليل البيانات غير قابل للكتابة!
    { $dir }
panic-unable-to-resolve-custom-data-directory =
    فشل في حل دليل بيانات المذكرات المخصصة!
    { $dir }
    
    تأكد من أن المسار موجود كدليل، أو إزالة إعداد
    `المذكرات. ata` لاستخدام مسار البيانات الافتراضي.
panic-unable-to-create-backup-directory =
    غير قادر على إنشاء دليل النسخ الاحتياطي!
    { $dir }
panic-backup-directory-is-a-file =
    يوجد دليل النسخ الاحتياطي كملف!
    { $dir }
panic-backup-directory-is-not-writable =
    دليل النسخ الاحتياطي غير قابل للكتابة!
    { $dir }
panic-database-file-is-not-writable =
    ملف قاعدة البيانات غير قابل للكتابة!
    { $file }
panic-failed-to-connect-to-database = فشل الاتصال بقاعدة البيانات
panic-failed-to-run-database-migrations =
    فشل تشغيل عمليات ترحيل قاعدة البيانات:
    { $error }
panic-failed-to-close-database-connection = فشل في إغلاق اتصال قاعدة البيانات
warn-failed-to-backup-database =
    فشل النسخ الاحتياطي لقاعدة البيانات:
    
    { $error }
prompt-install-webview-title = خطأ في WebView
prompt-install-webview-message =
    WebView *مطلوب* لهذا التطبيق إلى
    العمل وهو غير متوفر على هذا النظام!
    
    هل تريد تثبيته؟
error-failed-to-install-webview =
    فشل تثبيت WebView:
    
    { $error }
    
    الرجاء تثبيته يدويا.
panic-config-unable-to-create = غير قادر على إنشاء ملف التكوين
panic-config-is-not-a-file =
    مسار التكوين المتاح هو الدليل.
    يجب أن يكون ملف!
    { $path }
panic-config-is-not-writable =
    ملف التكوين غير قابل للكتابة!
    { $file }
prompt-config-error-title = خطأ في التكوين
prompt-config-error-message =
    فشل تحليل ملف الإعداد:
    
    { $error }
    
    إعادة تعيين ملف الإعداد؟
    (سيتم إنشاء نسخة احتياطية.)
panic-config-error =
    الرجاء إصلاح ملف التكوين
    يدوياً وإعادة تشغيل التطبيق.
panic-config-unable-to-backup = فشل في النسخ الاحتياطي لملف التكوين الحالي
panic-config-unable-to-reset = غير قادر على إعادة تعيين ملف التكوين
panic-config-parse-error = خطأ فادح أثناء تحليل ملف التكوين
error-config-write-error =
    فشل في كتابة ملف الإعداد:
    
    { $error }
panic-portpicker-error = فشل في العثور على منفذ مجاني لربط المذكرة!
error-invalid-server-url =
    رابط الخادم البعيد غير صالح:
    { $url }
    
    يجب أن يبدأ بـ "http".
    تحقق من الإعدادات.
panic-unable-to-find-memos-binary = غير قادر على العثور على خادم Memos الثنائي!
panic-log-config-write-error =
    فشل في كتابة ملف إعدادات السجل:
    { $file }
panic-log-config-reset-error =
    فشل في إعادة تعيين ملف إعدادات السجل:
    { $file }
    الرجاء حذفه وإعادة تشغيل التطبيق.
