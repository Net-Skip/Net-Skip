# Copyright © The Daybrite Project
# SPDX-License-Identifier: MPL-2.0
#
# All app-owned UI text. Settings and About labels come from Day's catalog.
app_title = Net Skip
about_tagline = Un navigateur web qui sait se faire oublier.

## Menus
menu_file = Fichier
menu_edit = Édition
menu_view = Présentation
menu_history = Historique
menu_bookmarks = Signets
menu_tabs = Onglets

## Tabs
new_tab = Nouvel onglet
new_private_tab = Nouvel onglet privé
private_tab = Onglet privé
close_tab = Fermer l’onglet
close_other_tabs = Fermer les autres onglets
duplicate_tab = Dupliquer l’onglet
reopen_closed_tab = Rouvrir l’onglet fermé
recently_closed = Fermés récemment
no_closed_tabs = Aucun onglet fermé récemment
show_all_tabs = Afficher tous les onglets
next_tab = Onglet suivant
previous_tab = Onglet précédent
tab_number = Onglet { $n }
last_tab = Dernier onglet
tab_count = { $n ->
    [one] { $n } onglet
   *[other] { $n } onglets
}
open_in_new_tab = Ouvrir dans un nouvel onglet
open_in_background_tab = Ouvrir dans un onglet en arrière-plan
open_location = Ouvrir une adresse…
private_explainer = Navigation privée : les pages que vous consultez dans cet onglet restent hors de votre historique, et l’onglet est oublié dès que vous le fermez.

## Navigation
back = Retour
forward = Suivant
reload = Recharger la page
stop_loading = Arrêter le chargement
home = Accueil
share = Partager
more = Plus
done = Terminé
edit = Modifier
cancel = Annuler
settings = Réglages

## The address bar
address_placeholder = Rechercher avec { $engine } ou saisir une adresse
secure_connection = Connexion sécurisée
clear_text = Effacer
search_with = Rechercher avec { $engine }
go_to_site = Aller sur le site
switch_to_tab = Aller à l’onglet · { $site }
fill_in_search = Reprendre dans la recherche

## Find in page and zoom
find_on_page = Rechercher sur la page…
find_placeholder = Rechercher sur la page
find_previous = Correspondance précédente
find_next = Correspondance suivante
find_none = Introuvable
find_count = { $current } sur { $total }
text_size = Taille du texte
desktop_site = Site pour ordinateur
request_desktop_site = Demander le site pour ordinateur
request_mobile_site = Demander le site mobile
hide_toolbar = Masquer la barre d’outils
actual_size_percent = Taille réelle ({ $percent } %)
page_menu = Menu de la page
show_toolbar = Afficher la barre d’outils
zoom_in = Zoom avant
zoom_out = Zoom arrière
actual_size = Taille réelle

## Bookmarks and history
show_library = Signets et historique
show_bookmarks = Afficher les signets
show_all_history = Afficher tout l’historique
bookmarks = Signets
history = Historique
add_bookmark = Ajouter un signet
remove_bookmark = Supprimer le signet
copy_link = Copier le lien
delete_history_item = Supprimer de l’historique
search_bookmarks = Rechercher dans les signets
search_history = Rechercher dans l’historique
bookmarks_empty = Ajoutez une page aux signets pour la retrouver ici et sur la page de démarrage.
history_empty = Les pages que vous consultez apparaissent ici.
no_matches = Aucun résultat pour cette recherche.
favorites = Favoris
favorites_empty = Vos signets apparaissent ici.
frequently_visited = Sites fréquents
today = Aujourd’hui
yesterday = Hier
monday = Lundi
tuesday = Mardi
wednesday = Mercredi
thursday = Jeudi
friday = Vendredi
saturday = Samedi
sunday = Dimanche
clear_history_button = Effacer…
clear_history_prompt = Effacer l’historique :
clear_last_hour = La dernière heure
clear_today = Aujourd’hui
clear_two_days = Aujourd’hui et hier
clear_all_history = Tout l’historique
clear_history_message = L’effacement retire de votre historique les pages que vous avez consultées.
delete = Supprimer

## Settings
settings_search = Recherche
settings_browsing = Navigation
settings_address_bar = Barre d’adresse
settings_privacy = Confidentialité
search_engine = Moteur de recherche
search_custom = Personnalisé
search_custom_template = Adresse de recherche
search_custom_placeholder = https://example.com/search?q=%s
search_custom_caption = L’adresse de la page de résultats, avec %s à la place des termes recherchés.
search_suggestions = Suggestions de recherche
search_suggestions_caption = Envoie au moteur de recherche ce que vous tapez dans la barre d’adresse, au fil de la saisie.
library_suggestions = Suggestions de Net Skip
library_suggestions_caption = Propose des pages de votre historique et de vos signets pendant la saisie.
settings_tabs = Onglets
address_at_top = Barre d’adresse en haut
address_at_top_caption = Sur un téléphone, affiche la barre d’adresse au-dessus de la page plutôt qu’en dessous.
open_links_background = Ouvrir les liens en arrière-plan
close_tabs = Fermer les onglets
close_tabs_manually = Manuellement
close_tabs_day = Après un jour
close_tabs_week = Après une semaine
close_tabs_month = Après un mois
home_page = Page d’accueil
home_page_placeholder = Page de démarrage
restore_tabs = Rouvrir les onglets de la dernière fois
show_frequent = Afficher les sites fréquemment consultés
full_address = Afficher l’adresse complète du site
upgrade_https = Passer en HTTPS
upgrade_https_caption = Charge de façon sécurisée les adresses http:// saisies quand le site le permet.
page_zoom = Zoom de la page
blocking = Publicités et traqueurs
blocking_level = Blocage
blocking_off = Désactivé
blocking_light = Léger
blocking_strong = Fort
blocking_aggressive = Agressif
blocking_caption = « Léger » bloque la plupart des traqueurs et quelques publicités. « Fort » bloque davantage de traqueurs et de publicités ; certains sites risquent de ne plus fonctionner. « Agressif » bloque presque tous les traqueurs et publicités ; des sites ne fonctionneront plus correctement.
blocking_update = Mettre à jour les listes
blocking_unavailable = Le blocage de contenu n’est pas disponible.
blocking_status_off = Le blocage est désactivé.
blocking_status_waiting = Téléchargement des listes de filtres…
blocking_status_ready = { $lists ->
    [one] { $lists } liste de filtres utilisée, tenue à jour.
   *[other] { $lists } listes de filtres utilisées, tenues à jour.
}
blocking_status_failed = Impossible de télécharger les listes de filtres. Net Skip réessaiera.
blocking_custom = Personnalisé
blocking_filter_lists = Listes de filtres
blocking_lists_caption = Activer ou désactiver une liste, ou modifier une option, fait passer le blocage en « Personnalisé ». Restaurez un préréglage pour repartir de zéro.
blocking_lists_general = Listes
blocking_lists_regional = Listes régionales
blocking_lists_added = Listes ajoutées
blocking_purpose_privacy = Traqueurs
blocking_purpose_ads = Publicités
blocking_purpose_cookies = Bandeaux de cookies
blocking_purpose_annoyances = Éléments gênants
blocking_purpose_social = Widgets sociaux
blocking_purpose_regional = Publicités régionales
blocking_purpose_security = Logiciels malveillants et arnaques
blocking_list_not_downloaded = Non téléchargée
blocking_list_failed = Échec du téléchargement
blocking_list_stale = Échec de la mise à jour ; dernière copie utilisée
blocking_list_updated_now = Mise à jour à l’instant
blocking_list_updated_minutes = { $n ->
    [one] Mise à jour il y a { $n } minute
   *[other] Mise à jour il y a { $n } minutes
}
blocking_list_updated_hours = { $n ->
    [one] Mise à jour il y a { $n } heure
   *[other] Mise à jour il y a { $n } heures
}
blocking_list_updated_days = { $n ->
    [one] Mise à jour il y a { $n } jour
   *[other] Mise à jour il y a { $n } jours
}
blocking_add_list = Ajouter une liste
blocking_add_list_placeholder = https://example.com/list.txt
blocking_add_list_caption = Les listes au format Adblock Plus ou hosts sont prises en charge. Les règles qui exécutent des scripts ou réécrivent les pages sont comptées mais pas appliquées.
blocking_add_checking = Vérification de la liste…
blocking_add_failed = Impossible d’ajouter cette liste : { $error }
blocking_add_done = { $rules ->
    [one] « { $title } » ajoutée : { $rules } règle.
   *[other] « { $title } » ajoutée : { $rules } règles.
}
blocking_add_unsupported = { $n ->
    [one] { $n } règle ne peut pas être appliquée.
   *[other] { $n } règles ne peuvent pas être appliquées.
}
blocking_remove_list = Supprimer
blocking_options = Options
blocking_document_blocking = Bloquer les pages des sites bloqués
blocking_document_blocking_caption = Les liens vers un site figurant sur une liste ne s’ouvrent pas.
blocking_same_site = Bloquer les requêtes du site lui-même
blocking_same_site_caption = Bloque les publicités qu’un site diffuse lui-même. Plus efficace, mais davantage de sites risquent de ne plus fonctionner.
blocking_restore_preset = Restaurer un préréglage
blocking_status_preparing = Préparation des listes de filtres…
blocking_status_kept = Impossible de télécharger { $lists }. Les listes précédentes restent utilisées.
blocking_reload_offer = Les listes ont changé. Rechargez la page pour les utiliser.
blocking_reload_page = Recharger la page
blocking_this_site = Blocage sur ce site
blocking_site_default = Utiliser le réglage par défaut ({ $level })
blocked_count = { $n ->
    [one] { $n } élément bloqué
   *[other] { $n } éléments bloqués
}
clear_history = Effacer l’historique
clear_history_confirm = Effacer tout l’historique de navigation ? Cette action est irréversible.
clear_website_data = Effacer les données des sites
clear_website_data_caption = Supprime les cookies, les caches et les données enregistrées par les sites, ce qui vous déconnecte des sites web.
clear_website_data_confirm = Supprimer tous les cookies et les données des sites ? Vous serez déconnecté des sites web.
clear_bookmarks = Supprimer tous les signets
clear_bookmarks_confirm = Supprimer tous les signets ? Cette action est irréversible.

## Downloads, links to other apps, sign-in and error pages
downloads = Téléchargements
download_confirm = Télécharger « { $name } » ?
download = Télécharger
search_downloads = Rechercher dans les téléchargements
downloads_empty = Les fichiers que vous téléchargez apparaissent ici.
download_progress = { $received } sur { $total }
download_failed = Échec
download_canceled = Annulé
clear_downloads = Effacer
remove_download = Retirer de la liste
open_app_title = Ouvrir dans une autre app ?
open_app_message = Ce lien s’ouvre en dehors de Net Skip : { $url }
open = Ouvrir
sign_in_title = Se connecter
sign_in_message = « { $host } » demande un nom et un mot de passe.
sign_in_failed = Ce nom ou ce mot de passe n’a pas été accepté.
user_name = Nom d’utilisateur
password = Mot de passe
sign_in = Se connecter
error_offline_title = Vous êtes hors ligne
error_offline_message = Connectez-vous à Internet, puis réessayez.
error_host_title = Site introuvable
error_host_message = Net Skip ne trouve pas le serveur « { $host } ».
error_timeout_title = Le site met trop de temps à répondre
error_timeout_message = « { $host } » n’a pas répondu à temps.
error_connection_title = Connexion impossible
error_connection_message = Net Skip n’a pas pu se connecter à « { $host } ».
error_secure_title = Connexion sécurisée impossible
error_secure_message = Net Skip n’a pas pu établir de connexion sécurisée avec « { $host } ».
error_certificate_title = Cette connexion n’est pas privée
error_certificate_message = Le certificat de sécurité de « { $host } » n’est pas fiable. Quelqu’un se fait peut-être passer pour ce site afin de voler vos informations.
error_crashed_title = Cette page a cessé de fonctionner
error_crashed_message = Un problème avec la page a entraîné sa fermeture.
error_other_title = Impossible d’ouvrir cette page
error_other_message = Net Skip n’a pas pu ouvrir la page de « { $host } ».
try_again = Réessayer
visit_anyway = Consulter le site quand même
visit_anyway_title = Consulter ce site web ?
visit_anyway_message = Votre connexion à « { $host } » ne sera pas privée.

## The user agent setting
settings_websites = Sites web
user_agent = Agent utilisateur
user_agent_caption = Ce que Net Skip annonce être aux sites web. Les pages se rechargent quand vous le modifiez.
user_agent_default = Par défaut
user_agent_engine = Vue web intégrée
user_agent_custom = Agent utilisateur personnalisé

## Permission prompts
# What the system shows when a web page asks for a device feature (Day.toml [permissions]).
permission_camera = Net Skip a besoin d’accéder à votre caméra afin que les sites web que vous consultez puissent l’utiliser, avec votre autorisation, pour les appels vidéo, la lecture de codes QR et la prise de photos.
permission_microphone = Net Skip a besoin d’accéder à votre microphone afin que les sites web que vous consultez puissent l’utiliser, avec votre autorisation, pour les appels audio et vidéo, la recherche vocale et l’enregistrement audio.
permission_location_when_in_use = Net Skip utilise votre position pour fournir des fonctionnalités géolocalisées aux sites web que vous consultez, telles que cartes, résultats de recherche locale et informations météo, lorsque vous leur en accordez l’autorisation.
permission_motion = Net Skip utilise les données de mouvement et d’orientation afin que les sites web que vous consultez puissent proposer des expériences interactives réagissant à la façon dont vous bougez et inclinez votre appareil, lorsque vous leur en accordez l’autorisation.
permission_NSSpeechRecognitionUsageDescription = Net Skip utilise la reconnaissance vocale afin que vous puissiez dicter des URL, des requêtes de recherche et des saisies de formulaires plutôt que de les taper.
