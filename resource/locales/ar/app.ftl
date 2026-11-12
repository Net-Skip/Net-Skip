# Copyright © The Daybrite Project
# SPDX-License-Identifier: MPL-2.0
#
# All app-owned UI text. Settings and About labels come from Day's catalog.
app_title = Net Skip
about_tagline = متصفح ويب لا يعترض طريقك.

## Menus
menu_file = ملف
menu_edit = تحرير
menu_view = عرض
menu_history = السجل
menu_bookmarks = الإشارات المرجعية
menu_tabs = علامات التبويب

## Tabs
new_tab = علامة تبويب جديدة
new_private_tab = علامة تبويب خاصة جديدة
private_tab = علامة تبويب خاصة
close_tab = إغلاق علامة التبويب
close_other_tabs = إغلاق علامات التبويب الأخرى
duplicate_tab = تكرار علامة التبويب
reopen_closed_tab = إعادة فتح علامة التبويب المغلقة
recently_closed = المغلقة مؤخرًا
no_closed_tabs = لا توجد علامات تبويب مغلقة مؤخرًا
show_all_tabs = إظهار كل علامات التبويب
next_tab = علامة التبويب التالية
previous_tab = علامة التبويب السابقة
tab_number = علامة التبويب { $n }
last_tab = علامة التبويب الأخيرة
tab_count = { $n ->
    [zero] لا توجد علامات تبويب
    [one] علامة تبويب واحدة
    [two] علامتا تبويب
    [few] { $n } علامات تبويب
    [many] { $n } علامة تبويب
   *[other] { $n } علامة تبويب
}
open_in_new_tab = فتح في علامة تبويب جديدة
open_in_background_tab = فتح في علامة تبويب في الخلفية
open_location = فتح عنوان…
private_explainer = التصفح الخاص: الصفحات التي تزورها في علامة التبويب هذه لا تُضاف إلى سجلك، وتُنسى علامة التبويب عند إغلاقها.

## Navigation
back = رجوع
forward = للأمام
reload = إعادة تحميل الصفحة
stop_loading = إيقاف التحميل
home = الرئيسية
share = مشاركة
more = المزيد
done = تم
edit = تعديل
cancel = إلغاء
settings = الإعدادات

## The address bar
address_placeholder = ابحث في { $engine } أو أدخل عنوانًا
secure_connection = اتصال آمن
clear_text = مسح
search_with = ابحث في { $engine }
go_to_site = الانتقال إلى الموقع
switch_to_tab = التبديل إلى علامة التبويب · { $site }
fill_in_search = إدراج في البحث

## Find in page and zoom
find_on_page = البحث في الصفحة…
find_placeholder = البحث في الصفحة
find_previous = المطابقة السابقة
find_next = المطابقة التالية
find_none = غير موجود
find_count = { $current } من { $total }
text_size = حجم النص
desktop_site = موقع سطح المكتب
request_desktop_site = طلب موقع سطح المكتب
request_mobile_site = طلب موقع الأجهزة المحمولة
hide_toolbar = إخفاء شريط الأدوات
actual_size_percent = الحجم الفعلي ({ $percent }%)
page_menu = قائمة الصفحة
show_toolbar = إظهار شريط الأدوات
zoom_in = تكبير
zoom_out = تصغير
actual_size = الحجم الفعلي

## Bookmarks and history
show_library = الإشارات المرجعية والسجل
show_bookmarks = إظهار الإشارات المرجعية
show_all_history = إظهار السجل بالكامل
bookmarks = الإشارات المرجعية
history = السجل
add_bookmark = إضافة إشارة مرجعية
remove_bookmark = إزالة الإشارة المرجعية
copy_link = نسخ الرابط
delete_history_item = إزالة من السجل
search_bookmarks = البحث في الإشارات المرجعية
search_history = البحث في السجل
bookmarks_empty = أضف إشارة مرجعية إلى صفحة لتجدها هنا وفي صفحة البداية.
history_empty = تظهر هنا الصفحات التي تزورها.
no_matches = لا شيء يطابق بحثك.
favorites = المفضلة
favorites_empty = تظهر إشاراتك المرجعية هنا.
frequently_visited = الأكثر زيارة
today = اليوم
yesterday = أمس
monday = الاثنين
tuesday = الثلاثاء
wednesday = الأربعاء
thursday = الخميس
friday = الجمعة
saturday = السبت
sunday = الأحد
clear_history_button = مسح…
clear_history_prompt = مسح السجل:
clear_last_hour = آخر ساعة
clear_today = اليوم
clear_two_days = اليوم وأمس
clear_all_history = السجل بالكامل
clear_history_message = يؤدي المسح إلى إزالة الصفحات التي زرتها من سجلك.
delete = حذف

## Settings
settings_search = البحث
settings_browsing = التصفح
settings_address_bar = شريط العناوين
settings_privacy = الخصوصية
search_engine = محرك البحث
search_custom = مخصص
search_custom_template = عنوان البحث
search_custom_placeholder = https://example.com/search?q=%s
search_custom_caption = عنوان صفحة النتائج، مع %s في موضع عبارة البحث.
search_suggestions = اقتراحات البحث
search_suggestions_caption = يرسل ما تكتبه في شريط العناوين إلى محرك البحث أثناء الكتابة.
library_suggestions = اقتراحات Net Skip
library_suggestions_caption = يقترح صفحات من سجلك وإشاراتك المرجعية أثناء الكتابة.
settings_tabs = علامات التبويب
address_at_top = شريط العناوين في الأعلى
address_at_top_caption = على الهاتف، يُظهر شريط العناوين فوق الصفحة بدلاً من أسفلها.
open_links_background = فتح الروابط في الخلفية
close_tabs = إغلاق علامات التبويب
close_tabs_manually = يدويًا
close_tabs_day = بعد يوم واحد
close_tabs_week = بعد أسبوع واحد
close_tabs_month = بعد شهر واحد
home_page = الصفحة الرئيسية
home_page_placeholder = صفحة البداية
restore_tabs = إعادة فتح علامات التبويب من المرة السابقة
show_frequent = إظهار المواقع الأكثر زيارة
full_address = إظهار عنوان الموقع بالكامل
upgrade_https = الترقية إلى HTTPS
upgrade_https_caption = يحمّل عناوين ‎http://‎ التي تكتبها بأمان عندما يسمح الموقع بذلك.
page_zoom = تكبير الصفحة
blocking = الإعلانات وأدوات التتبع
blocking_level = الحظر
blocking_off = إيقاف
blocking_light = خفيف
blocking_strong = قوي
blocking_aggressive = شديد
blocking_caption = يحظر المستوى «خفيف» معظم أدوات التتبع وبعض الإعلانات. ويحظر «قوي» مزيدًا من أدوات التتبع والإعلانات، وقد تتعطل بعض المواقع. أما «شديد» فيحظر كل أدوات التتبع والإعلانات تقريبًا، وستتعطل بعض المواقع حتمًا.
blocking_update = تحديث القوائم الآن
blocking_unavailable = حظر المحتوى غير متاح.
blocking_status_off = الحظر متوقف.
blocking_status_waiting = جارٍ تنزيل قوائم التصفية…
blocking_status_ready = { $lists ->
    [zero] لا تُستخدم أي قائمة تصفية.
    [one] تُستخدم قائمة تصفية واحدة يجري تحديثها باستمرار.
    [two] تُستخدم قائمتا تصفية يجري تحديثهما باستمرار.
    [few] تُستخدم { $lists } قوائم تصفية يجري تحديثها باستمرار.
    [many] تُستخدم { $lists } قائمة تصفية يجري تحديثها باستمرار.
   *[other] تُستخدم { $lists } قائمة تصفية يجري تحديثها باستمرار.
}
blocking_status_failed = تعذّر تنزيل قوائم التصفية. سيحاول Net Skip مرة أخرى.
blocking_custom = مخصّص
blocking_filter_lists = قوائم التصفية
blocking_lists_caption = يؤدي تشغيل قائمة أو إيقافها، أو تغيير أحد الخيارات، إلى جعل الحظر «مخصّص». استعِد إعدادًا مسبقًا للبدء من جديد.
blocking_lists_general = القوائم
blocking_lists_regional = القوائم الإقليمية
blocking_lists_added = القوائم المضافة
blocking_purpose_privacy = أدوات التتبع
blocking_purpose_ads = الإعلانات
blocking_purpose_cookies = إشعارات ملفات تعريف الارتباط
blocking_purpose_annoyances = العناصر المزعجة
blocking_purpose_social = أدوات التواصل الاجتماعي
blocking_purpose_regional = الإعلانات الإقليمية
blocking_purpose_security = البرامج الضارة والاحتيال
blocking_list_not_downloaded = لم يتم تنزيلها
blocking_list_failed = فشل التنزيل
blocking_list_stale = تعذّر التحديث؛ تُستخدم آخر نسخة
blocking_list_updated_now = تم التحديث للتو
blocking_list_updated_minutes = { $n ->
    [zero] تم التحديث للتو
    [one] تم التحديث قبل دقيقة واحدة
    [two] تم التحديث قبل دقيقتين
    [few] تم التحديث قبل { $n } دقائق
    [many] تم التحديث قبل { $n } دقيقة
   *[other] تم التحديث قبل { $n } دقيقة
}
blocking_list_updated_hours = { $n ->
    [zero] تم التحديث قبل أقل من ساعة
    [one] تم التحديث قبل ساعة واحدة
    [two] تم التحديث قبل ساعتين
    [few] تم التحديث قبل { $n } ساعات
    [many] تم التحديث قبل { $n } ساعة
   *[other] تم التحديث قبل { $n } ساعة
}
blocking_list_updated_days = { $n ->
    [zero] تم التحديث اليوم
    [one] تم التحديث قبل يوم واحد
    [two] تم التحديث قبل يومين
    [few] تم التحديث قبل { $n } أيام
    [many] تم التحديث قبل { $n } يومًا
   *[other] تم التحديث قبل { $n } يوم
}
blocking_add_list = إضافة قائمة
blocking_add_list_placeholder = https://example.com/list.txt
blocking_add_list_caption = تعمل القوائم بتنسيق Adblock Plus أو hosts. تُحتسب القواعد التي تشغّل نصوصًا برمجية أو تعيد كتابة الصفحات، لكنها لا تُطبَّق.
blocking_add_checking = جارٍ فحص القائمة…
blocking_add_failed = تعذّرت إضافة هذه القائمة: { $error }
blocking_add_done = { $rules ->
    [zero] تمت إضافة «{ $title }»: لا توجد قواعد.
    [one] تمت إضافة «{ $title }»: قاعدة واحدة.
    [two] تمت إضافة «{ $title }»: قاعدتان.
    [few] تمت إضافة «{ $title }»: { $rules } قواعد.
    [many] تمت إضافة «{ $title }»: { $rules } قاعدة.
   *[other] تمت إضافة «{ $title }»: { $rules } قاعدة.
}
blocking_add_unsupported = { $n ->
    [zero] لا توجد قواعد يتعذّر تطبيقها.
    [one] يتعذّر تطبيق قاعدة واحدة.
    [two] يتعذّر تطبيق قاعدتين.
    [few] يتعذّر تطبيق { $n } قواعد.
    [many] يتعذّر تطبيق { $n } قاعدة.
   *[other] يتعذّر تطبيق { $n } قاعدة.
}
blocking_remove_list = إزالة
blocking_options = الخيارات
blocking_document_blocking = حظر صفحات المواقع المحظورة
blocking_document_blocking_caption = لا تُفتح الروابط إلى موقع مدرج في قائمة.
blocking_same_site = حظر طلبات الموقع نفسه
blocking_same_site_caption = يحظر الإعلانات التي يقدّمها الموقع بنفسه. يحظر المزيد، لكن تتعطل مواقع أكثر.
blocking_restore_preset = استعادة الإعداد المسبق
blocking_status_preparing = جارٍ تجهيز قوائم التصفية…
blocking_status_kept = تعذّر تنزيل { $lists }. تبقى القوائم السابقة قيد الاستخدام.
blocking_reload_offer = تغيّرت القوائم. أعِد تحميل الصفحة لاستخدامها.
blocking_reload_page = إعادة تحميل الصفحة
blocking_this_site = الحظر على هذا الموقع
blocking_site_default = استخدام الإعداد الافتراضي ({ $level })
blocked_count = { $n ->
    [zero] لم يُحظر أي عنصر
    [one] تم حظر عنصر واحد
    [two] تم حظر عنصرين
    [few] تم حظر { $n } عناصر
    [many] تم حظر { $n } عنصرًا
   *[other] تم حظر { $n } عنصر
}
clear_history = مسح السجل
clear_history_confirm = مسح سجل التصفح بالكامل؟ لا يمكن التراجع عن ذلك.
clear_website_data = مسح بيانات المواقع
clear_website_data_caption = يزيل ملفات تعريف الارتباط وذاكرة التخزين المؤقت وبيانات المواقع المحفوظة، فيسجّل خروجك من المواقع.
clear_website_data_confirm = إزالة كل ملفات تعريف الارتباط وبيانات المواقع؟ سيُسجَّل خروجك من المواقع.
clear_bookmarks = حذف كل الإشارات المرجعية
clear_bookmarks_confirm = حذف كل الإشارات المرجعية؟ لا يمكن التراجع عن ذلك.

## Downloads, links to other apps, sign-in and error pages
downloads = التنزيلات
download_confirm = تنزيل ”{ $name }“؟
download = تنزيل
search_downloads = البحث في التنزيلات
downloads_empty = تظهر هنا الملفات التي تنزّلها.
download_progress = { $received } من { $total }
download_failed = فشل
download_canceled = أُلغي
clear_downloads = مسح
remove_download = إزالة من القائمة
open_app_title = الفتح في تطبيق آخر؟
open_app_message = يُفتح هذا الرابط خارج Net Skip: { $url }
open = فتح
sign_in_title = تسجيل الدخول
sign_in_message = يطلب ”{ $host }“ اسمًا وكلمة سر.
sign_in_failed = لم يُقبل هذا الاسم أو كلمة السر.
user_name = اسم المستخدم
password = كلمة السر
sign_in = تسجيل الدخول
error_offline_title = أنت غير متصل
error_offline_message = اتصل بالإنترنت ثم حاول مرة أخرى.
error_host_title = تعذّر العثور على الموقع
error_host_message = لا يستطيع Net Skip العثور على الخادم ”{ $host }“.
error_timeout_title = استغرق الموقع وقتًا طويلًا للرد
error_timeout_message = لم يردّ ”{ $host }“ في الوقت المناسب.
error_connection_title = تعذّر الاتصال
error_connection_message = لم يتمكن Net Skip من الاتصال بـ ”{ $host }“.
error_secure_title = تعذّر الاتصال بشكل آمن
error_secure_message = لم يتمكن Net Skip من إنشاء اتصال آمن بـ ”{ $host }“.
error_certificate_title = هذا الاتصال غير خاص
error_certificate_message = شهادة الأمان الخاصة بـ ”{ $host }“ غير موثوقة. ربما ينتحل شخص ما صفة الموقع لسرقة معلوماتك.
error_crashed_title = توقفت هذه الصفحة عن العمل
error_crashed_message = تسببت مشكلة في الصفحة في إغلاقها.
error_other_title = تعذّر فتح هذه الصفحة
error_other_message = لم يتمكن Net Skip من فتح الصفحة على ”{ $host }“.
try_again = حاول مرة أخرى
visit_anyway = زيارة الموقع على أي حال
visit_anyway_title = زيارة هذا الموقع؟
visit_anyway_message = لن يكون اتصالك بـ ”{ $host }“ خاصًا.

## The user agent setting
settings_websites = المواقع
user_agent = وكيل المستخدم
user_agent_caption = ما يعرّف به Net Skip نفسه للمواقع. يُعاد تحميل الصفحات عند تغييره.
user_agent_default = الافتراضي
user_agent_engine = عرض الويب المضمّن
user_agent_custom = وكيل مستخدم مخصص

## Permission prompts
# What the system shows when a web page asks for a device feature (Day.toml [permissions]).
permission_camera = يحتاج Net Skip إلى الوصول إلى الكاميرا حتى تتمكن المواقع التي تزورها من استخدامها لمكالمات الفيديو ومسح رموز QR والتقاط الصور عند منحها الإذن.
permission_microphone = يحتاج Net Skip إلى الوصول إلى الميكروفون حتى تتمكن المواقع التي تزورها من استخدامه للمكالمات الصوتية والمرئية والبحث الصوتي وتسجيل الصوت عند منحها الإذن.
permission_location_when_in_use = يستخدم Net Skip موقعك لتوفير ميزات تعتمد على الموقع للمواقع التي تزورها، مثل الخرائط ونتائج البحث المحلية ومعلومات الطقس عند منحها الإذن.
permission_motion = يستخدم Net Skip بيانات الحركة والاتجاه حتى تتمكن المواقع التي تزورها من تقديم تجارب تفاعلية تستجيب لطريقة تحريك وإمالة جهازك عند منحها الإذن.
permission_NSSpeechRecognitionUsageDescription = يستخدم Net Skip التعرف على الكلام حتى تتمكن من إملاء عناوين URL واستفسارات البحث ومدخلات النماذج صوتياً بدلاً من الكتابة.
