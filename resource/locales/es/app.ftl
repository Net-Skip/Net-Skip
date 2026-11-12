# Copyright © The Daybrite Project
# SPDX-License-Identifier: MPL-2.0
#
# All app-owned UI text. Settings and About labels come from Day's catalog.
app_title = Net Skip
about_tagline = Un navegador web que no estorba.

## Menus
menu_file = Archivo
menu_edit = Edición
menu_view = Visualización
menu_history = Historial
menu_bookmarks = Marcadores
menu_tabs = Pestañas

## Tabs
new_tab = Nueva pestaña
new_private_tab = Nueva pestaña privada
private_tab = Pestaña privada
close_tab = Cerrar pestaña
close_other_tabs = Cerrar otras pestañas
duplicate_tab = Duplicar pestaña
reopen_closed_tab = Reabrir pestaña cerrada
recently_closed = Cerradas recientemente
no_closed_tabs = No hay pestañas cerradas recientemente
show_all_tabs = Mostrar todas las pestañas
next_tab = Pestaña siguiente
previous_tab = Pestaña anterior
tab_number = Pestaña { $n }
last_tab = Última pestaña
tab_count = { $n ->
    [one] { $n } pestaña
   *[other] { $n } pestañas
}
open_in_new_tab = Abrir en pestaña nueva
open_in_background_tab = Abrir en pestaña en segundo plano
open_location = Abrir ubicación…
private_explainer = Navegación privada: las páginas que visitas en esta pestaña no se guardan en tu historial, y la pestaña se olvida al cerrarla.

## Navigation
back = Atrás
forward = Adelante
reload = Recargar página
stop_loading = Detener carga
home = Inicio
share = Compartir
more = Más
done = Listo
edit = Editar
cancel = Cancelar
settings = Ajustes

## The address bar
address_placeholder = Busca en { $engine } o escribe una dirección
secure_connection = Conexión segura
clear_text = Borrar
search_with = Buscar en { $engine }
go_to_site = Ir al sitio
switch_to_tab = Ir a la pestaña · { $site }
fill_in_search = Pasar a la búsqueda

## Find in page and zoom
find_on_page = Buscar en la página…
find_placeholder = Buscar en la página
find_previous = Coincidencia anterior
find_next = Siguiente coincidencia
find_none = No encontrado
find_count = { $current } de { $total }
text_size = Tamaño del texto
desktop_site = Sitio de escritorio
request_desktop_site = Solicitar sitio web de escritorio
request_mobile_site = Solicitar sitio web móvil
hide_toolbar = Ocultar barra de herramientas
actual_size_percent = Tamaño real ({ $percent } %)
page_menu = Menú de la página
show_toolbar = Mostrar barra de herramientas
zoom_in = Ampliar
zoom_out = Reducir
actual_size = Tamaño real

## Bookmarks and history
show_library = Marcadores e historial
show_bookmarks = Mostrar marcadores
show_all_history = Mostrar todo el historial
bookmarks = Marcadores
history = Historial
add_bookmark = Añadir marcador
remove_bookmark = Eliminar marcador
copy_link = Copiar enlace
delete_history_item = Eliminar del historial
search_bookmarks = Buscar en marcadores
search_history = Buscar en el historial
bookmarks_empty = Añade una página a marcadores para tenerla aquí y en la página de inicio.
history_empty = Aquí aparecen las páginas que visitas.
no_matches = Nada coincide con tu búsqueda.
favorites = Favoritos
favorites_empty = Aquí aparecen tus marcadores.
frequently_visited = Visitados con frecuencia
today = Hoy
yesterday = Ayer
monday = Lunes
tuesday = Martes
wednesday = Miércoles
thursday = Jueves
friday = Viernes
saturday = Sábado
sunday = Domingo
clear_history_button = Borrar…
clear_history_prompt = Borrar el historial de:
clear_last_hour = La última hora
clear_today = Hoy
clear_two_days = Hoy y ayer
clear_all_history = Todo el historial
clear_history_message = Al borrar, se eliminan de tu historial las páginas que has visitado.
delete = Eliminar

## Settings
settings_search = Búsqueda
settings_browsing = Navegación
settings_address_bar = Barra de direcciones
settings_privacy = Privacidad
search_engine = Motor de búsqueda
search_custom = Personalizado
search_custom_template = Dirección de búsqueda
search_custom_placeholder = https://example.com/search?q=%s
search_custom_caption = La dirección de la página de resultados, con %s donde va la búsqueda.
search_suggestions = Sugerencias de búsqueda
search_suggestions_caption = Envía al motor de búsqueda lo que escribes en la barra de direcciones mientras lo escribes.
library_suggestions = Sugerencias de Net Skip
library_suggestions_caption = Propone páginas de tu historial y tus marcadores mientras escribes.
settings_tabs = Pestañas
address_at_top = Barra de direcciones arriba
address_at_top_caption = En un teléfono, muestra la barra de direcciones encima de la página en lugar de debajo.
open_links_background = Abrir enlaces en segundo plano
close_tabs = Cerrar pestañas
close_tabs_manually = Manualmente
close_tabs_day = Después de un día
close_tabs_week = Después de una semana
close_tabs_month = Después de un mes
home_page = Página principal
home_page_placeholder = Página de inicio
restore_tabs = Reabrir las pestañas de la última vez
show_frequent = Mostrar los sitios visitados con frecuencia
full_address = Mostrar la dirección completa del sitio web
upgrade_https = Actualizar a HTTPS
upgrade_https_caption = Carga de forma segura las direcciones http:// que escribes cuando el sitio lo permite.
page_zoom = Zoom de página
blocking = Anuncios y rastreadores
blocking_level = Bloqueo
blocking_off = Desactivado
blocking_light = Ligero
blocking_strong = Fuerte
blocking_aggressive = Agresivo
blocking_caption = «Ligero» bloquea la mayoría de los rastreadores y algunos anuncios. «Fuerte» bloquea más rastreadores y anuncios; algunos sitios podrían dejar de funcionar. «Agresivo» bloquea casi todos los rastreadores y anuncios; habrá sitios que dejen de funcionar.
blocking_update = Actualizar listas ahora
blocking_unavailable = El bloqueo de contenido no está disponible.
blocking_status_off = El bloqueo está desactivado.
blocking_status_waiting = Descargando listas de filtros…
blocking_status_ready = { $lists ->
    [one] Usando { $lists } lista de filtros, siempre actualizada.
   *[other] Usando { $lists } listas de filtros, siempre actualizadas.
}
blocking_status_failed = No se han podido descargar las listas de filtros. Net Skip lo volverá a intentar.
blocking_custom = Personalizado
blocking_filter_lists = Listas de filtros
blocking_lists_caption = Activar o desactivar una lista, o cambiar una opción, pone el bloqueo en «Personalizado». Restaura un ajuste predefinido para empezar de nuevo.
blocking_lists_general = Listas
blocking_lists_regional = Listas regionales
blocking_lists_added = Listas añadidas
blocking_purpose_privacy = Rastreadores
blocking_purpose_ads = Anuncios
blocking_purpose_cookies = Avisos de cookies
blocking_purpose_annoyances = Molestias
blocking_purpose_social = Widgets sociales
blocking_purpose_regional = Anuncios regionales
blocking_purpose_security = Malware y estafas
blocking_list_not_downloaded = Sin descargar
blocking_list_failed = Error de descarga
blocking_list_stale = Error al actualizar; se usa la última copia
blocking_list_updated_now = Actualizada hace un momento
blocking_list_updated_minutes = { $n ->
    [one] Actualizada hace { $n } minuto
   *[other] Actualizada hace { $n } minutos
}
blocking_list_updated_hours = { $n ->
    [one] Actualizada hace { $n } hora
   *[other] Actualizada hace { $n } horas
}
blocking_list_updated_days = { $n ->
    [one] Actualizada hace { $n } día
   *[other] Actualizada hace { $n } días
}
blocking_add_list = Añadir lista
blocking_add_list_placeholder = https://example.com/list.txt
blocking_add_list_caption = Funcionan las listas en formato Adblock Plus o hosts. Las reglas que ejecutan scripts o reescriben páginas se cuentan, pero no se aplican.
blocking_add_checking = Comprobando la lista…
blocking_add_failed = No se ha podido añadir esta lista: { $error }
blocking_add_done = { $rules ->
    [one] Se ha añadido «{ $title }»: { $rules } regla.
   *[other] Se ha añadido «{ $title }»: { $rules } reglas.
}
blocking_add_unsupported = { $n ->
    [one] { $n } regla no se puede aplicar.
   *[other] { $n } reglas no se pueden aplicar.
}
blocking_remove_list = Eliminar
blocking_options = Opciones
blocking_document_blocking = Bloquear páginas de sitios bloqueados
blocking_document_blocking_caption = Los enlaces a un sitio de una lista no se abren.
blocking_same_site = Bloquear solicitudes del propio sitio
blocking_same_site_caption = Bloquea los anuncios que sirve el propio sitio. Bloquea más, pero más sitios dejan de funcionar.
blocking_restore_preset = Restaurar ajuste predefinido
blocking_status_preparing = Preparando listas de filtros…
blocking_status_kept = No se ha podido descargar { $lists }. Se siguen usando las listas anteriores.
blocking_reload_offer = Las listas han cambiado. Recarga la página para usarlas.
blocking_reload_page = Recargar página
blocking_this_site = Bloqueo en este sitio
blocking_site_default = Usar predeterminado ({ $level })
blocked_count = { $n ->
    [one] { $n } elemento bloqueado
   *[other] { $n } elementos bloqueados
}
clear_history = Borrar historial
clear_history_confirm = ¿Borrar todo el historial de navegación? No se puede deshacer.
clear_website_data = Borrar datos de sitios web
clear_website_data_caption = Elimina las cookies, las cachés y los datos guardados de los sitios, y cierra tu sesión en los sitios web.
clear_website_data_confirm = ¿Eliminar todas las cookies y los datos de sitios web? Se cerrará tu sesión en los sitios web.
clear_bookmarks = Eliminar todos los marcadores
clear_bookmarks_confirm = ¿Eliminar todos los marcadores? No se puede deshacer.

## Downloads, links to other apps, sign-in and error pages
downloads = Descargas
download_confirm = ¿Descargar «{ $name }»?
download = Descargar
search_downloads = Buscar en descargas
downloads_empty = Aquí aparecen los archivos que descargas.
download_progress = { $received } de { $total }
download_failed = Error
download_canceled = Cancelada
clear_downloads = Vaciar
remove_download = Quitar de la lista
open_app_title = ¿Abrir en otra app?
open_app_message = Este enlace se abre fuera de Net Skip: { $url }
open = Abrir
sign_in_title = Iniciar sesión
sign_in_message = «{ $host }» pide un nombre y una contraseña.
sign_in_failed = No se aceptó ese nombre o esa contraseña.
user_name = Nombre de usuario
password = Contraseña
sign_in = Iniciar sesión
error_offline_title = No tienes conexión
error_offline_message = Conéctate a internet y vuelve a intentarlo.
error_host_title = No se encuentra el sitio
error_host_message = Net Skip no encuentra el servidor «{ $host }».
error_timeout_title = El sitio tarda demasiado en responder
error_timeout_message = «{ $host }» no respondió a tiempo.
error_connection_title = No se puede conectar
error_connection_message = Net Skip no pudo conectarse a «{ $host }».
error_secure_title = No se puede conectar de forma segura
error_secure_message = Net Skip no pudo establecer una conexión segura con «{ $host }».
error_certificate_title = Esta conexión no es privada
error_certificate_message = El certificado de seguridad de «{ $host }» no es de confianza. Alguien podría estar suplantando el sitio para robar tu información.
error_crashed_title = Esta página dejó de funcionar
error_crashed_message = Un problema con la página hizo que se cerrara.
error_other_title = No se puede abrir esta página
error_other_message = Net Skip no pudo abrir la página de «{ $host }».
try_again = Reintentar
visit_anyway = Visitar el sitio de todos modos
visit_anyway_title = ¿Visitar este sitio web?
visit_anyway_message = Tu conexión con «{ $host }» no será privada.

## The user agent setting
settings_websites = Sitios web
user_agent = Agente de usuario
user_agent_caption = Cómo se identifica Net Skip ante los sitios web. Las páginas se vuelven a cargar al cambiarlo.
user_agent_default = Predeterminado
user_agent_engine = Vista web integrada
user_agent_custom = Agente de usuario personalizado

## Permission prompts
# What the system shows when a web page asks for a device feature (Day.toml [permissions]).
permission_camera = Net Skip necesita acceder a tu cámara para que los sitios web que visitas puedan usarla para videollamadas, escanear códigos QR y hacer fotos cuando les concedes permiso.
permission_microphone = Net Skip necesita acceder a tu micrófono para que los sitios web que visitas puedan usarlo en llamadas de voz y vídeo, búsqueda por voz y grabación de audio cuando les concedes permiso.
permission_location_when_in_use = Net Skip usa tu ubicación para ofrecer funciones basadas en la ubicación a los sitios web que visitas, como mapas, resultados de búsqueda local e información meteorológica cuando les concedes permiso.
permission_motion = Net Skip utiliza datos de movimiento y orientación para que los sitios web que visitas puedan ofrecer experiencias interactivas que respondan a cómo mueves e inclinas el dispositivo cuando les concedes permiso.
permission_NSSpeechRecognitionUsageDescription = Net Skip usa el reconocimiento de voz para que puedas dictar URLs, consultas de búsqueda y entradas de formularios con la voz en lugar de escribir.
