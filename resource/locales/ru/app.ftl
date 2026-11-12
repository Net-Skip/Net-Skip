# Copyright © The Daybrite Project
# SPDX-License-Identifier: MPL-2.0
#
# All app-owned UI text. Settings and About labels come from Day's catalog.
app_title = Net Skip
about_tagline = Браузер, который не мешает.

## Menus
menu_file = Файл
menu_edit = Правка
menu_view = Вид
menu_history = История
menu_bookmarks = Закладки
menu_tabs = Вкладки

## Tabs
new_tab = Новая вкладка
new_private_tab = Новая частная вкладка
private_tab = Частная вкладка
close_tab = Закрыть вкладку
close_other_tabs = Закрыть другие вкладки
duplicate_tab = Дублировать вкладку
reopen_closed_tab = Открыть закрытую вкладку
recently_closed = Недавно закрытые
no_closed_tabs = Нет недавно закрытых вкладок
show_all_tabs = Показать все вкладки
next_tab = Следующая вкладка
previous_tab = Предыдущая вкладка
tab_number = Вкладка { $n }
last_tab = Последняя вкладка
tab_count = { $n ->
    [one] { $n } вкладка
    [few] { $n } вкладки
    [many] { $n } вкладок
   *[other] { $n } вкладки
}
open_in_new_tab = Открыть в новой вкладке
open_in_background_tab = Открыть в фоновой вкладке
open_location = Открыть адрес…
private_explainer = Частный доступ: страницы, открытые в этой вкладке, не попадают в историю, а сама вкладка забывается, как только вы её закроете.

## Navigation
back = Назад
forward = Вперёд
reload = Перезагрузить страницу
stop_loading = Остановить загрузку
home = Главная
share = Поделиться
more = Ещё
done = Готово
edit = Изменить
cancel = Отмена
settings = Настройки

## The address bar
address_placeholder = Поиск в { $engine } или адрес сайта
secure_connection = Защищённое соединение
clear_text = Очистить
search_with = Искать в { $engine }
go_to_site = Перейти на сайт
switch_to_tab = Перейти на вкладку · { $site }
fill_in_search = Подставить в поиск

## Find in page and zoom
find_on_page = Найти на странице…
find_placeholder = Найти на странице
find_previous = Предыдущее совпадение
find_next = Следующее совпадение
find_none = Не найдено
find_count = { $current } из { $total }
text_size = Размер текста
desktop_site = Версия для ПК
request_desktop_site = Запросить настольный веб-сайт
request_mobile_site = Запросить мобильный веб-сайт
hide_toolbar = Скрыть панель инструментов
actual_size_percent = Реальный размер ({ $percent } %)
page_menu = Меню страницы
show_toolbar = Показать панель инструментов
zoom_in = Увеличить
zoom_out = Уменьшить
actual_size = Реальный размер

## Bookmarks and history
show_library = Закладки и история
show_bookmarks = Показать закладки
show_all_history = Показать всю историю
bookmarks = Закладки
history = История
add_bookmark = Добавить закладку
remove_bookmark = Удалить закладку
copy_link = Скопировать ссылку
delete_history_item = Удалить из истории
search_bookmarks = Поиск в закладках
search_history = Поиск в истории
bookmarks_empty = Добавьте страницу в закладки, чтобы она была здесь и на начальной странице.
history_empty = Здесь появятся страницы, которые вы посещаете.
no_matches = По вашему запросу ничего не найдено.
favorites = Избранное
favorites_empty = Здесь появятся ваши закладки.
frequently_visited = Часто посещаемые
today = Сегодня
yesterday = Вчера
monday = Понедельник
tuesday = Вторник
wednesday = Среда
thursday = Четверг
friday = Пятница
saturday = Суббота
sunday = Воскресенье
clear_history_button = Очистить…
clear_history_prompt = Очистить историю за:
clear_last_hour = Последний час
clear_today = Сегодня
clear_two_days = Сегодня и вчера
clear_all_history = Всё время
clear_history_message = Посещённые страницы будут удалены из истории.
delete = Удалить

## Settings
settings_search = Поиск
settings_browsing = Просмотр
settings_address_bar = Адресная строка
settings_privacy = Конфиденциальность
search_engine = Поисковая система
search_custom = Своя
search_custom_template = Адрес поиска
search_custom_placeholder = https://example.com/search?q=%s
search_custom_caption = Адрес страницы результатов, где %s стоит на месте запроса.
search_suggestions = Поисковые подсказки
search_suggestions_caption = Отправляет поисковой системе то, что вы вводите в адресную строку, прямо во время ввода.
library_suggestions = Подсказки Net Skip
library_suggestions_caption = Предлагает страницы из истории и закладок во время ввода.
settings_tabs = Вкладки
address_at_top = Адресная строка вверху
address_at_top_caption = На телефоне показывает адресную строку над страницей, а не под ней.
open_links_background = Открывать ссылки в фоне
close_tabs = Закрывать вкладки
close_tabs_manually = Вручную
close_tabs_day = Через день
close_tabs_week = Через неделю
close_tabs_month = Через месяц
home_page = Домашняя страница
home_page_placeholder = Начальная страница
restore_tabs = Открывать вкладки прошлого сеанса
show_frequent = Показывать часто посещаемые сайты
full_address = Показывать полный адрес сайта
upgrade_https = Обновлять до HTTPS
upgrade_https_caption = Загружает введённые адреса http:// по защищённому соединению, если сайт это поддерживает.
page_zoom = Масштаб страницы
blocking = Реклама и трекеры
blocking_level = Блокировка
blocking_off = Выключена
blocking_light = Лёгкая
blocking_strong = Строгая
blocking_aggressive = Агрессивная
blocking_caption = «Лёгкая» блокирует большинство трекеров и часть рекламы. «Строгая» блокирует больше трекеров и рекламы; некоторые сайты могут работать неправильно. «Агрессивная» блокирует почти все трекеры и рекламу; сайты будут работать неправильно.
blocking_update = Обновить списки
blocking_unavailable = Блокировка контента недоступна.
blocking_status_off = Блокировка выключена.
blocking_status_waiting = Загрузка списков фильтров…
blocking_status_ready = { $lists ->
    [one] Используется { $lists } список фильтров, он регулярно обновляется.
    [few] Используются { $lists } списка фильтров, они регулярно обновляются.
    [many] Используются { $lists } списков фильтров, они регулярно обновляются.
   *[other] Используется { $lists } списка фильтров, они регулярно обновляются.
}
blocking_status_failed = Не удалось загрузить списки фильтров. Net Skip повторит попытку.
blocking_custom = Своя
blocking_filter_lists = Списки фильтров
blocking_lists_caption = Если включить или выключить список или изменить параметр, блокировка переключится на «Своя». Чтобы начать заново, восстановите предустановку.
blocking_lists_general = Списки
blocking_lists_regional = Региональные списки
blocking_lists_added = Добавленные списки
blocking_purpose_privacy = Трекеры
blocking_purpose_ads = Реклама
blocking_purpose_cookies = Уведомления о cookie
blocking_purpose_annoyances = Раздражающие элементы
blocking_purpose_social = Виджеты соцсетей
blocking_purpose_regional = Региональная реклама
blocking_purpose_security = Вредоносное ПО и мошенничество
blocking_list_not_downloaded = Не загружен
blocking_list_failed = Ошибка загрузки
blocking_list_stale = Не удалось обновить; используется последняя копия
blocking_list_updated_now = Обновлён только что
blocking_list_updated_minutes = { $n ->
    [one] Обновлён { $n } минуту назад
    [few] Обновлён { $n } минуты назад
    [many] Обновлён { $n } минут назад
   *[other] Обновлён { $n } минуты назад
}
blocking_list_updated_hours = { $n ->
    [one] Обновлён { $n } час назад
    [few] Обновлён { $n } часа назад
    [many] Обновлён { $n } часов назад
   *[other] Обновлён { $n } часа назад
}
blocking_list_updated_days = { $n ->
    [one] Обновлён { $n } день назад
    [few] Обновлён { $n } дня назад
    [many] Обновлён { $n } дней назад
   *[other] Обновлён { $n } дня назад
}
blocking_add_list = Добавить список
blocking_add_list_placeholder = https://example.com/list.txt
blocking_add_list_caption = Подходят списки в формате Adblock Plus или hosts. Правила, которые запускают скрипты или переписывают страницы, учитываются, но не применяются.
blocking_add_checking = Проверка списка…
blocking_add_failed = Не удалось добавить список: { $error }
blocking_add_done = { $rules ->
    [one] Добавлен «{ $title }»: { $rules } правило.
    [few] Добавлен «{ $title }»: { $rules } правила.
    [many] Добавлен «{ $title }»: { $rules } правил.
   *[other] Добавлен «{ $title }»: { $rules } правила.
}
blocking_add_unsupported = { $n ->
    [one] { $n } правило нельзя применить.
    [few] { $n } правила нельзя применить.
    [many] { $n } правил нельзя применить.
   *[other] { $n } правила нельзя применить.
}
blocking_remove_list = Удалить
blocking_options = Параметры
blocking_document_blocking = Блокировать страницы заблокированных сайтов
blocking_document_blocking_caption = Ссылки на сайты из списков не открываются.
blocking_same_site = Блокировать собственные запросы сайта
blocking_same_site_caption = Блокирует рекламу, которую сайт показывает сам. Блокирует больше, но чаще ломает сайты.
blocking_restore_preset = Восстановить предустановку
blocking_status_preparing = Подготовка списков фильтров…
blocking_status_kept = Не удалось загрузить: { $lists }. Используются прежние списки.
blocking_reload_offer = Списки изменились. Перезагрузите страницу, чтобы применить их.
blocking_reload_page = Перезагрузить страницу
blocking_this_site = Блокировка на этом сайте
blocking_site_default = По умолчанию ({ $level })
blocked_count = { $n ->
    [one] Заблокирован { $n } элемент
    [few] Заблокировано { $n } элемента
    [many] Заблокировано { $n } элементов
   *[other] Заблокировано { $n } элемента
}
clear_history = Очистить историю
clear_history_confirm = Удалить всю историю браузера? Это действие нельзя отменить.
clear_website_data = Удалить данные сайтов
clear_website_data_caption = Удаляет файлы cookie, кеш и сохранённые данные сайтов, поэтому вы выйдете из своих учётных записей на сайтах.
clear_website_data_confirm = Удалить все файлы cookie и данные сайтов? Вы выйдете из учётных записей на сайтах.
clear_bookmarks = Удалить все закладки
clear_bookmarks_confirm = Удалить все закладки? Это действие нельзя отменить.

## Downloads, links to other apps, sign-in and error pages
downloads = Загрузки
download_confirm = Загрузить «{ $name }»?
download = Загрузить
search_downloads = Поиск в загрузках
downloads_empty = Здесь появятся файлы, которые вы загружаете.
download_progress = { $received } из { $total }
download_failed = Ошибка
download_canceled = Отменено
clear_downloads = Очистить
remove_download = Убрать из списка
open_app_title = Открыть в другом приложении?
open_app_message = Эта ссылка открывается вне Net Skip: { $url }
open = Открыть
sign_in_title = Вход
sign_in_message = Сайт «{ $host }» запрашивает имя и пароль.
sign_in_failed = Имя или пароль не приняты.
user_name = Имя пользователя
password = Пароль
sign_in = Войти
error_offline_title = Нет подключения к интернету
error_offline_message = Подключитесь к интернету и повторите попытку.
error_host_title = Сайт не найден
error_host_message = Net Skip не может найти сервер «{ $host }».
error_timeout_title = Сайт слишком долго не отвечает
error_timeout_message = «{ $host }» не ответил вовремя.
error_connection_title = Не удаётся подключиться
error_connection_message = Net Skip не удалось подключиться к «{ $host }».
error_secure_title = Не удаётся установить безопасное соединение
error_secure_message = Net Skip не удалось установить безопасное соединение с «{ $host }».
error_certificate_title = Это соединение не является приватным
error_certificate_message = Сертификат безопасности «{ $host }» не является доверенным. Возможно, кто-то выдаёт себя за этот сайт, чтобы украсть ваши данные.
error_crashed_title = Страница перестала работать
error_crashed_message = Из-за проблемы со страницей она была закрыта.
error_other_title = Не удаётся открыть страницу
error_other_message = Net Skip не удалось открыть страницу на «{ $host }».
try_again = Повторить
visit_anyway = Всё равно перейти на сайт
visit_anyway_title = Перейти на этот сайт?
visit_anyway_message = Ваше соединение с «{ $host }» не будет приватным.

## The user agent setting
settings_websites = Сайты
user_agent = User-Agent
user_agent_caption = Как Net Skip представляется сайтам. При изменении страницы загружаются заново.
user_agent_default = По умолчанию
user_agent_engine = Встроенный веб-просмотр
user_agent_custom = Свой User-Agent

## Permission prompts
# What the system shows when a web page asks for a device feature (Day.toml [permissions]).
permission_camera = Net Skip требуется доступ к камере, чтобы посещаемые сайты могли с вашего разрешения использовать её для видеозвонков, сканирования QR-кодов и съёмки фотографий.
permission_microphone = Net Skip требуется доступ к микрофону, чтобы посещаемые сайты могли с вашего разрешения использовать его для голосовых и видеозвонков, голосового поиска и записи аудио.
permission_location_when_in_use = Net Skip использует ваше местоположение для предоставления функций на основе местоположения сайтам, которые вы посещаете, например карт, результатов локального поиска и сведений о погоде с вашего разрешения.
permission_motion = Net Skip использует данные движения и ориентации, чтобы посещаемые сайты могли с вашего разрешения предлагать интерактивные функции, реагирующие на то, как вы двигаете и наклоняете устройство.
permission_NSSpeechRecognitionUsageDescription = Net Skip использует распознавание речи, чтобы вы могли диктовать URL-адреса, поисковые запросы и данные форм голосом вместо ввода текста.
