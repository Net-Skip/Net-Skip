# Copyright © The Daybrite Project
# SPDX-License-Identifier: MPL-2.0
#
# All app-owned UI text. Settings and About labels come from Day's catalog.
app_title = Net Skip
about_tagline = Un browser web che non ti intralcia.

## Menus
menu_file = File
menu_edit = Modifica
menu_view = Vista
menu_history = Cronologia
menu_bookmarks = Segnalibri
menu_tabs = Schede

## Tabs
new_tab = Nuova scheda
new_private_tab = Nuova scheda privata
private_tab = Scheda privata
close_tab = Chiudi scheda
close_other_tabs = Chiudi altre schede
duplicate_tab = Duplica scheda
reopen_closed_tab = Riapri scheda chiusa
recently_closed = Chiuse di recente
no_closed_tabs = Nessuna scheda chiusa di recente
show_all_tabs = Mostra tutte le schede
next_tab = Scheda successiva
previous_tab = Scheda precedente
tab_number = Scheda { $n }
last_tab = Ultima scheda
tab_count = { $n ->
    [one] { $n } scheda
   *[other] { $n } schede
}
open_in_new_tab = Apri in nuova scheda
open_in_background_tab = Apri in una scheda in background
open_location = Apri indirizzo…
private_explainer = Navigazione privata: le pagine che visiti in questa scheda restano fuori dalla cronologia e la scheda viene dimenticata quando la chiudi.

## Navigation
back = Indietro
forward = Avanti
reload = Ricarica pagina
stop_loading = Interrompi caricamento
home = Home
share = Condividi
more = Altro
done = Fine
edit = Modifica
cancel = Annulla
settings = Impostazioni

## The address bar
address_placeholder = Cerca con { $engine } o inserisci un indirizzo
secure_connection = Connessione sicura
clear_text = Cancella
search_with = Cerca con { $engine }
go_to_site = Vai al sito
switch_to_tab = Passa alla scheda · { $site }
fill_in_search = Inserisci nella ricerca

## Find in page and zoom
find_on_page = Trova nella pagina…
find_placeholder = Trova nella pagina
find_previous = Risultato precedente
find_next = Risultato successivo
find_none = Non trovato
find_count = { $current } di { $total }
text_size = Dimensioni testo
desktop_site = Sito desktop
request_desktop_site = Richiedi sito web desktop
request_mobile_site = Richiedi sito web mobile
hide_toolbar = Nascondi barra strumenti
actual_size_percent = Dimensioni reali ({ $percent }%)
page_menu = Menu pagina
show_toolbar = Mostra barra strumenti
zoom_in = Ingrandisci
zoom_out = Riduci
actual_size = Dimensioni reali

## Bookmarks and history
show_library = Segnalibri e cronologia
show_bookmarks = Mostra segnalibri
show_all_history = Mostra tutta la cronologia
bookmarks = Segnalibri
history = Cronologia
add_bookmark = Aggiungi segnalibro
remove_bookmark = Rimuovi segnalibro
copy_link = Copia link
delete_history_item = Rimuovi dalla cronologia
search_bookmarks = Cerca nei segnalibri
search_history = Cerca nella cronologia
bookmarks_empty = Aggiungi una pagina ai segnalibri per ritrovarla qui e nella pagina di avvio.
history_empty = Qui compaiono le pagine che visiti.
no_matches = Nessun risultato per la tua ricerca.
favorites = Preferiti
favorites_empty = Qui compaiono i tuoi segnalibri.
frequently_visited = Visitati spesso
today = Oggi
yesterday = Ieri
monday = Lunedì
tuesday = Martedì
wednesday = Mercoledì
thursday = Giovedì
friday = Venerdì
saturday = Sabato
sunday = Domenica
clear_history_button = Cancella…
clear_history_prompt = Cancella la cronologia di:
clear_last_hour = L’ultima ora
clear_today = Oggi
clear_two_days = Oggi e ieri
clear_all_history = Tutta la cronologia
clear_history_message = La cancellazione rimuove dalla cronologia le pagine che hai visitato.
delete = Elimina

## Settings
settings_search = Ricerca
settings_browsing = Navigazione
settings_address_bar = Barra degli indirizzi
settings_privacy = Privacy
search_engine = Motore di ricerca
search_custom = Personalizzato
search_custom_template = Indirizzo di ricerca
search_custom_placeholder = https://example.com/search?q=%s
search_custom_caption = L’indirizzo della pagina dei risultati, con %s al posto dei termini cercati.
search_suggestions = Suggerimenti di ricerca
search_suggestions_caption = Invia al motore di ricerca ciò che digiti nella barra degli indirizzi, mentre lo digiti.
library_suggestions = Suggerimenti di Net Skip
library_suggestions_caption = Propone pagine dalla cronologia e dai segnalibri mentre digiti.
settings_tabs = Schede
address_at_top = Barra degli indirizzi in alto
address_at_top_caption = Su un telefono, mostra la barra degli indirizzi sopra la pagina anziché sotto.
open_links_background = Apri i link in background
close_tabs = Chiudi schede
close_tabs_manually = Manualmente
close_tabs_day = Dopo un giorno
close_tabs_week = Dopo una settimana
close_tabs_month = Dopo un mese
home_page = Pagina iniziale
home_page_placeholder = Pagina di avvio
restore_tabs = Riapri le schede dell’ultima volta
show_frequent = Mostra i siti visitati spesso
full_address = Mostra l’indirizzo completo del sito
upgrade_https = Aggiorna a HTTPS
upgrade_https_caption = Carica in modo sicuro gli indirizzi http:// digitati, quando il sito lo consente.
page_zoom = Zoom pagina
blocking = Pubblicità e tracker
blocking_level = Blocco
blocking_off = Disattivato
blocking_light = Leggero
blocking_strong = Forte
blocking_aggressive = Aggressivo
blocking_caption = «Leggero» blocca la maggior parte dei tracker e alcune pubblicità. «Forte» blocca più tracker e pubblicità; alcuni siti potrebbero non funzionare. «Aggressivo» blocca quasi tutti i tracker e le pubblicità; i siti non funzioneranno correttamente.
blocking_update = Aggiorna elenchi ora
blocking_unavailable = Il blocco dei contenuti non è disponibile.
blocking_status_off = Il blocco è disattivato.
blocking_status_waiting = Download degli elenchi di filtri…
blocking_status_ready = { $lists ->
    [one] { $lists } elenco di filtri in uso, sempre aggiornato.
   *[other] { $lists } elenchi di filtri in uso, sempre aggiornati.
}
blocking_status_failed = Impossibile scaricare gli elenchi di filtri. Net Skip riproverà.
blocking_custom = Personalizzato
blocking_filter_lists = Elenchi di filtri
blocking_lists_caption = Attivare o disattivare un elenco, o modificare un’opzione, imposta il blocco su «Personalizzato». Ripristina una preimpostazione per ricominciare.
blocking_lists_general = Elenchi
blocking_lists_regional = Elenchi regionali
blocking_lists_added = Elenchi aggiunti
blocking_purpose_privacy = Tracker
blocking_purpose_ads = Pubblicità
blocking_purpose_cookies = Avvisi sui cookie
blocking_purpose_annoyances = Elementi fastidiosi
blocking_purpose_social = Widget social
blocking_purpose_regional = Pubblicità regionali
blocking_purpose_security = Malware e truffe
blocking_list_not_downloaded = Non scaricato
blocking_list_failed = Download non riuscito
blocking_list_stale = Aggiornamento non riuscito; in uso l’ultima copia
blocking_list_updated_now = Aggiornato ora
blocking_list_updated_minutes = { $n ->
    [one] Aggiornato { $n } minuto fa
   *[other] Aggiornato { $n } minuti fa
}
blocking_list_updated_hours = { $n ->
    [one] Aggiornato { $n } ora fa
   *[other] Aggiornato { $n } ore fa
}
blocking_list_updated_days = { $n ->
    [one] Aggiornato { $n } giorno fa
   *[other] Aggiornato { $n } giorni fa
}
blocking_add_list = Aggiungi elenco
blocking_add_list_placeholder = https://example.com/list.txt
blocking_add_list_caption = Funzionano gli elenchi in formato Adblock Plus o hosts. Le regole che eseguono script o riscrivono le pagine vengono conteggiate ma non applicate.
blocking_add_checking = Verifica dell’elenco…
blocking_add_failed = Impossibile aggiungere questo elenco: { $error }
blocking_add_done = { $rules ->
    [one] «{ $title }» aggiunto: { $rules } regola.
   *[other] «{ $title }» aggiunto: { $rules } regole.
}
blocking_add_unsupported = { $n ->
    [one] { $n } regola non può essere applicata.
   *[other] { $n } regole non possono essere applicate.
}
blocking_remove_list = Rimuovi
blocking_options = Opzioni
blocking_document_blocking = Blocca le pagine dei siti bloccati
blocking_document_blocking_caption = I link a un sito presente in un elenco non si aprono.
blocking_same_site = Blocca le richieste del sito stesso
blocking_same_site_caption = Blocca la pubblicità servita dal sito stesso. Blocca di più, ma più siti smettono di funzionare.
blocking_restore_preset = Ripristina preimpostazione
blocking_status_preparing = Preparazione degli elenchi di filtri…
blocking_status_kept = Impossibile scaricare { $lists }. Restano in uso gli elenchi precedenti.
blocking_reload_offer = Gli elenchi sono cambiati. Ricarica la pagina per usarli.
blocking_reload_page = Ricarica pagina
blocking_this_site = Blocco su questo sito
blocking_site_default = Usa predefinito ({ $level })
blocked_count = { $n ->
    [one] { $n } elemento bloccato
   *[other] { $n } elementi bloccati
}
clear_history = Cancella cronologia
clear_history_confirm = Cancellare tutta la cronologia di navigazione? L’operazione non può essere annullata.
clear_website_data = Cancella dati dei siti web
clear_website_data_caption = Rimuove cookie, cache e dati salvati dai siti, e ti disconnette dai siti web.
clear_website_data_confirm = Rimuovere tutti i cookie e i dati dei siti web? Verrai disconnesso dai siti web.
clear_bookmarks = Elimina tutti i segnalibri
clear_bookmarks_confirm = Eliminare tutti i segnalibri? L’operazione non può essere annullata.

## Downloads, links to other apps, sign-in and error pages
downloads = Download
download_confirm = Scaricare “{ $name }”?
download = Scarica
search_downloads = Cerca nei download
downloads_empty = Qui compaiono i file che scarichi.
download_progress = { $received } di { $total }
download_failed = Non riuscito
download_canceled = Annullato
clear_downloads = Svuota
remove_download = Rimuovi dall’elenco
open_app_title = Aprire in un’altra app?
open_app_message = Questo link si apre fuori da Net Skip: { $url }
open = Apri
sign_in_title = Accedi
sign_in_message = “{ $host }” richiede un nome e una password.
sign_in_failed = Il nome o la password non sono stati accettati.
user_name = Nome utente
password = Password
sign_in = Accedi
error_offline_title = Sei offline
error_offline_message = Connettiti a internet e riprova.
error_host_title = Sito non trovato
error_host_message = Net Skip non trova il server “{ $host }”.
error_timeout_title = Il sito impiega troppo tempo a rispondere
error_timeout_message = “{ $host }” non ha risposto in tempo.
error_connection_title = Impossibile connettersi
error_connection_message = Net Skip non è riuscito a connettersi a “{ $host }”.
error_secure_title = Impossibile connettersi in modo sicuro
error_secure_message = Net Skip non è riuscito a stabilire una connessione sicura con “{ $host }”.
error_certificate_title = Questa connessione non è privata
error_certificate_message = Il certificato di sicurezza di “{ $host }” non è attendibile. Qualcuno potrebbe fingersi il sito per rubare le tue informazioni.
error_crashed_title = Questa pagina ha smesso di funzionare
error_crashed_message = Un problema con la pagina ne ha causato la chiusura.
error_other_title = Impossibile aprire questa pagina
error_other_message = Net Skip non è riuscito ad aprire la pagina di “{ $host }”.
try_again = Riprova
visit_anyway = Visita comunque il sito
visit_anyway_title = Visitare questo sito web?
visit_anyway_message = La tua connessione a “{ $host }” non sarà privata.

## The user agent setting
settings_websites = Siti web
user_agent = User agent
user_agent_caption = Come Net Skip si presenta ai siti web. Le pagine vengono ricaricate quando lo cambi.
user_agent_default = Predefinito
user_agent_engine = Vista web integrata
user_agent_custom = User agent personalizzato

## Permission prompts
# What the system shows when a web page asks for a device feature (Day.toml [permissions]).
permission_camera = Net Skip necessita dell’accesso alla tua fotocamera affinché i siti web che visiti possano usarla per videochiamate, scansione di codici QR e scatto di foto quando concedi loro l’autorizzazione.
permission_microphone = Net Skip necessita dell’accesso al microfono affinché i siti web che visiti possano usarlo per chiamate vocali e video, ricerca vocale e registrazione audio quando concedi loro l’autorizzazione.
permission_location_when_in_use = Net Skip utilizza la tua posizione per fornire funzionalità basate sulla posizione ai siti web che visiti, come mappe, risultati di ricerca locale e informazioni meteo quando concedi loro l’autorizzazione.
permission_motion = Net Skip utilizza i dati di movimento e orientamento affinché i siti web che visiti possano offrire esperienze interattive che rispondono al modo in cui muovi e inclini il tuo dispositivo quando concedi loro l’autorizzazione.
permission_NSSpeechRecognitionUsageDescription = Net Skip utilizza il riconoscimento vocale per consentirti di dettare URL, query di ricerca e dati nei moduli con la voce invece di digitarli.
