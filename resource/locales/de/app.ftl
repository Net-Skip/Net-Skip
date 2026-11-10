# Copyright © The Daybrite Project
# SPDX-License-Identifier: MPL-2.0
#
# All app-owned UI text. Settings and About labels come from Day's catalog.
app_title = Net Skip
about_tagline = Ein Webbrowser, der sich zurückhält.

## Menus
menu_file = Datei
menu_edit = Bearbeiten
menu_view = Darstellung
menu_history = Verlauf
menu_bookmarks = Lesezeichen
menu_tabs = Tabs

## Tabs
new_tab = Neuer Tab
new_private_tab = Neuer privater Tab
private_tab = Privater Tab
close_tab = Tab schließen
close_other_tabs = Andere Tabs schließen
duplicate_tab = Tab duplizieren
reopen_closed_tab = Geschlossenen Tab wieder öffnen
recently_closed = Zuletzt geschlossen
no_closed_tabs = Keine zuletzt geschlossenen Tabs
show_all_tabs = Alle Tabs anzeigen
next_tab = Nächster Tab
previous_tab = Vorheriger Tab
tab_number = Tab { $n }
last_tab = Letzter Tab
tab_count = { $n ->
    [one] { $n } Tab
   *[other] { $n } Tabs
}
open_in_new_tab = In neuem Tab öffnen
open_in_background_tab = In Hintergrund-Tab öffnen
open_location = Adresse öffnen …
private_explainer = Privates Surfen: Seiten, die du in diesem Tab besuchst, erscheinen nicht in deinem Verlauf, und der Tab wird vergessen, sobald du ihn schließt.

## Navigation
back = Zurück
forward = Vorwärts
reload = Seite neu laden
stop_loading = Laden stoppen
home = Startseite
share = Teilen
more = Mehr
done = Fertig
edit = Bearbeiten
cancel = Abbrechen
settings = Einstellungen

## The address bar
address_placeholder = Mit { $engine } suchen oder Adresse eingeben
secure_connection = Sichere Verbindung
clear_text = Löschen
search_with = Mit { $engine } suchen
go_to_site = Website öffnen
switch_to_tab = Zum Tab wechseln · { $site }
fill_in_search = In die Suche übernehmen

## Find in page and zoom
find_on_page = Auf Seite suchen …
find_placeholder = Auf Seite suchen
find_previous = Vorheriger Treffer
find_next = Nächster Treffer
find_none = Nicht gefunden
find_count = { $current } von { $total }
text_size = Textgröße
desktop_site = Desktop-Website
request_desktop_site = Desktop-Website anfordern
request_mobile_site = Mobile Website anfordern
hide_toolbar = Symbolleiste ausblenden
actual_size_percent = Originalgröße ({ $percent } %)
page_menu = Seitenmenü
show_toolbar = Symbolleiste einblenden
zoom_in = Vergrößern
zoom_out = Verkleinern
actual_size = Originalgröße

## Bookmarks and history
show_library = Lesezeichen und Verlauf
show_bookmarks = Lesezeichen anzeigen
show_all_history = Gesamten Verlauf anzeigen
bookmarks = Lesezeichen
history = Verlauf
add_bookmark = Lesezeichen hinzufügen
remove_bookmark = Lesezeichen entfernen
copy_link = Link kopieren
delete_history_item = Aus Verlauf entfernen
search_bookmarks = Lesezeichen durchsuchen
search_history = Verlauf durchsuchen
bookmarks_empty = Setze ein Lesezeichen für eine Seite, um sie hier und auf der Startseite zu finden.
history_empty = Seiten, die du besuchst, erscheinen hier.
no_matches = Nichts passt zu deiner Suche.
favorites = Favoriten
favorites_empty = Deine Lesezeichen erscheinen hier.
frequently_visited = Häufig besucht
today = Heute
yesterday = Gestern
monday = Montag
tuesday = Dienstag
wednesday = Mittwoch
thursday = Donnerstag
friday = Freitag
saturday = Samstag
sunday = Sonntag
clear_history_button = Löschen …
clear_history_prompt = Verlauf löschen:
clear_last_hour = Letzte Stunde
clear_today = Heute
clear_two_days = Heute und gestern
clear_all_history = Gesamter Verlauf
clear_history_message = Beim Löschen werden die besuchten Seiten aus deinem Verlauf entfernt.
delete = Löschen

## Settings
settings_search = Suche
settings_browsing = Surfen
settings_address_bar = Adressleiste
settings_privacy = Datenschutz
search_engine = Suchmaschine
search_custom = Eigene
search_custom_template = Suchadresse
search_custom_placeholder = https://example.com/search?q=%s
search_custom_caption = Die Adresse der Ergebnisseite, mit %s an der Stelle des Suchbegriffs.
search_suggestions = Suchvorschläge
search_suggestions_caption = Sendet deine Eingabe in der Adressleiste schon beim Tippen an die Suchmaschine.
library_suggestions = Vorschläge von Net Skip
library_suggestions_caption = Schlägt beim Tippen Seiten aus deinem Verlauf und deinen Lesezeichen vor.
settings_tabs = Tabs
address_at_top = Adressleiste oben
address_at_top_caption = Zeigt die Adressleiste auf einem Smartphone über der Seite statt darunter.
open_links_background = Links im Hintergrund öffnen
close_tabs = Tabs schließen
close_tabs_manually = Manuell
close_tabs_day = Nach einem Tag
close_tabs_week = Nach einer Woche
close_tabs_month = Nach einem Monat
home_page = Homepage
home_page_placeholder = Startseite
restore_tabs = Tabs vom letzten Mal wieder öffnen
show_frequent = Häufig besuchte Websites anzeigen
full_address = Vollständige Websiteadresse anzeigen
upgrade_https = Auf HTTPS aktualisieren
upgrade_https_caption = Lädt eingetippte http://-Adressen sicher, wenn die Website es zulässt.
page_zoom = Seitenzoom
blocking = Werbung und Tracker
blocking_level = Blockierung
blocking_off = Aus
blocking_light = Leicht
blocking_strong = Stark
blocking_aggressive = Aggressiv
blocking_caption = „Leicht“ blockiert die meisten Tracker und etwas Werbung. „Stark“ blockiert mehr Tracker und Werbung; manche Websites funktionieren dann womöglich nicht richtig. „Aggressiv“ blockiert fast alle Tracker und Werbung; Websites funktionieren dann nicht mehr richtig.
blocking_update = Listen jetzt aktualisieren
blocking_unavailable = Das Blockieren von Inhalten ist nicht verfügbar.
blocking_status_off = Die Blockierung ist ausgeschaltet.
blocking_status_waiting = Filterlisten werden geladen …
blocking_status_ready = { $lists ->
    [one] { $lists } Filterliste aktiv, wird automatisch aktualisiert.
   *[other] { $lists } Filterlisten aktiv, werden automatisch aktualisiert.
}
blocking_status_failed = Die Filterlisten konnten nicht geladen werden. Net Skip versucht es erneut.
blocking_custom = Angepasst
blocking_filter_lists = Filterlisten
blocking_lists_caption = Wenn du eine Liste ein- oder ausschaltest oder eine Option änderst, wechselt die Blockierung zu „Angepasst“. Stelle eine Voreinstellung wieder her, um neu zu beginnen.
blocking_lists_general = Listen
blocking_lists_regional = Regionale Listen
blocking_lists_added = Hinzugefügte Listen
blocking_purpose_privacy = Tracker
blocking_purpose_ads = Werbung
blocking_purpose_cookies = Cookie-Hinweise
blocking_purpose_annoyances = Störende Elemente
blocking_purpose_social = Social-Media-Widgets
blocking_purpose_regional = Regionale Werbung
blocking_purpose_security = Malware und Betrug
blocking_list_not_downloaded = Nicht geladen
blocking_list_failed = Laden fehlgeschlagen
blocking_list_stale = Aktualisierung fehlgeschlagen; letzte Kopie wird verwendet
blocking_list_updated_now = Gerade aktualisiert
blocking_list_updated_minutes = { $n ->
    [one] Vor { $n } Minute aktualisiert
   *[other] Vor { $n } Minuten aktualisiert
}
blocking_list_updated_hours = { $n ->
    [one] Vor { $n } Stunde aktualisiert
   *[other] Vor { $n } Stunden aktualisiert
}
blocking_list_updated_days = { $n ->
    [one] Vor { $n } Tag aktualisiert
   *[other] Vor { $n } Tagen aktualisiert
}
blocking_add_list = Liste hinzufügen
blocking_add_list_placeholder = https://example.com/list.txt
blocking_add_list_caption = Listen im Format Adblock Plus oder hosts funktionieren. Regeln, die Skripte ausführen oder Seiten umschreiben, werden gezählt, aber nicht angewendet.
blocking_add_checking = Liste wird geprüft …
blocking_add_failed = Die Liste konnte nicht hinzugefügt werden: { $error }
blocking_add_done = { $rules ->
    [one] „{ $title }“ hinzugefügt: { $rules } Regel.
   *[other] „{ $title }“ hinzugefügt: { $rules } Regeln.
}
blocking_add_unsupported = { $n ->
    [one] { $n } Regel kann nicht angewendet werden.
   *[other] { $n } Regeln können nicht angewendet werden.
}
blocking_remove_list = Entfernen
blocking_options = Optionen
blocking_document_blocking = Seiten blockierter Websites sperren
blocking_document_blocking_caption = Links zu einer Website auf einer Liste öffnen sich nicht.
blocking_same_site = Eigene Anfragen der Website blockieren
blocking_same_site_caption = Blockiert Werbung, die eine Website selbst ausliefert. Erfasst mehr, aber mehr Websites funktionieren nicht richtig.
blocking_restore_preset = Voreinstellung wiederherstellen
blocking_status_preparing = Filterlisten werden vorbereitet …
blocking_status_kept = Laden fehlgeschlagen: { $lists }. Die bisherigen Listen bleiben aktiv.
blocking_reload_offer = Die Listen haben sich geändert. Lade die Seite neu, um sie zu verwenden.
blocking_reload_page = Seite neu laden
blocking_this_site = Blockieren auf dieser Website
blocking_site_default = Standard verwenden ({ $level })
blocked_count = { $n ->
    [one] { $n } Element blockiert
   *[other] { $n } Elemente blockiert
}
clear_history = Verlauf löschen
clear_history_confirm = Gesamten Browserverlauf löschen? Das kann nicht rückgängig gemacht werden.
clear_website_data = Websitedaten löschen
clear_website_data_caption = Entfernt Cookies, Caches und gespeicherte Websitedaten und meldet dich damit bei Websites ab.
clear_website_data_confirm = Alle Cookies und Websitedaten entfernen? Du wirst bei Websites abgemeldet.
clear_bookmarks = Alle Lesezeichen löschen
clear_bookmarks_confirm = Alle Lesezeichen löschen? Das kann nicht rückgängig gemacht werden.

## Permission prompts
# What the system shows when a web page asks for a device feature (Day.toml [permissions]).
permission_camera = Net Skip benötigt Zugriff auf deine Kamera, damit besuchte Websites sie mit deiner Erlaubnis für Videoanrufe, das Scannen von QR-Codes und das Aufnehmen von Fotos verwenden können.
permission_microphone = Net Skip benötigt Zugriff auf dein Mikrofon, damit besuchte Websites es mit deiner Erlaubnis für Sprach- und Videoanrufe, Sprachsuche und Audioaufnahmen verwenden können.
permission_location_when_in_use = Net Skip verwendet deinen Standort, um besuchten Websites standortbezogene Funktionen wie Karten, lokale Suchergebnisse und Wetterinformationen bereitzustellen, sofern du dies erlaubst.
permission_motion = Net Skip nutzt Bewegungs- und Orientierungsdaten, damit besuchte Websites mit deiner Erlaubnis interaktive Erlebnisse bieten können, die auf das Bewegen und Neigen deines Geräts reagieren.
permission_NSSpeechRecognitionUsageDescription = Net Skip verwendet Spracherkennung, damit du URLs, Suchanfragen und Formulareingaben per Sprache diktieren kannst, statt sie einzutippen.
