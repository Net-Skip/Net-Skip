# Copyright © The Daybrite Project
# SPDX-License-Identifier: MPL-2.0
#
# All app-owned UI text. Settings and About labels come from Day's catalog.
app_title = Net Skip
about_tagline = Um navegador web que não atrapalha.

## Menus
menu_file = Arquivo
menu_edit = Editar
menu_view = Visualizar
menu_history = Histórico
menu_bookmarks = Favoritos
menu_tabs = Abas

## Tabs
new_tab = Nova aba
new_private_tab = Nova aba privada
private_tab = Aba privada
close_tab = Fechar aba
close_other_tabs = Fechar outras abas
duplicate_tab = Duplicar aba
reopen_closed_tab = Reabrir aba fechada
recently_closed = Fechadas recentemente
no_closed_tabs = Nenhuma aba fechada recentemente
show_all_tabs = Mostrar todas as abas
next_tab = Próxima aba
previous_tab = Aba anterior
tab_number = Aba { $n }
last_tab = Última aba
tab_count = { $n ->
    [one] { $n } aba
   *[other] { $n } abas
}
open_in_new_tab = Abrir em nova aba
open_in_background_tab = Abrir em aba em segundo plano
open_location = Abrir endereço…
private_explainer = Navegação privada: as páginas que você visita nesta aba ficam fora do seu histórico, e a aba é esquecida quando você a fecha.

## Navigation
back = Voltar
forward = Avançar
reload = Recarregar página
stop_loading = Parar carregamento
home = Início
share = Compartilhar
more = Mais
done = Concluído
edit = Editar
cancel = Cancelar
settings = Ajustes

## The address bar
address_placeholder = Pesquise no { $engine } ou digite um endereço
secure_connection = Conexão segura
clear_text = Limpar
search_with = Pesquisar no { $engine }
go_to_site = Ir para o site
switch_to_tab = Ir para a aba · { $site }
fill_in_search = Levar para a pesquisa

## Find in page and zoom
find_on_page = Buscar na página…
find_placeholder = Buscar na página
find_previous = Ocorrência anterior
find_next = Próxima ocorrência
find_none = Não encontrado
find_count = { $current } de { $total }
text_size = Tamanho do texto
desktop_site = Site para computador
request_desktop_site = Solicitar site para computador
request_mobile_site = Solicitar site para celular
hide_toolbar = Ocultar barra de ferramentas
actual_size_percent = Tamanho real ({ $percent }%)
page_menu = Menu da página
show_toolbar = Mostrar barra de ferramentas
zoom_in = Ampliar
zoom_out = Reduzir
actual_size = Tamanho real

## Bookmarks and history
show_library = Favoritos e histórico
show_bookmarks = Mostrar favoritos
show_all_history = Mostrar todo o histórico
bookmarks = Favoritos
history = Histórico
add_bookmark = Adicionar aos favoritos
remove_bookmark = Remover dos favoritos
copy_link = Copiar link
delete_history_item = Remover do histórico
search_bookmarks = Buscar nos favoritos
search_history = Buscar no histórico
bookmarks_empty = Adicione uma página aos favoritos para tê-la aqui e na página de início.
history_empty = As páginas que você visita aparecem aqui.
no_matches = Nada corresponde à sua pesquisa.
favorites = Favoritos
favorites_empty = Seus favoritos aparecem aqui.
frequently_visited = Visitados com frequência
today = Hoje
yesterday = Ontem
monday = Segunda-feira
tuesday = Terça-feira
wednesday = Quarta-feira
thursday = Quinta-feira
friday = Sexta-feira
saturday = Sábado
sunday = Domingo
clear_history_button = Limpar…
clear_history_prompt = Limpar o histórico de:
clear_last_hour = Última hora
clear_today = Hoje
clear_two_days = Hoje e ontem
clear_all_history = Todo o histórico
clear_history_message = Limpar remove do seu histórico as páginas que você visitou.
delete = Apagar

## Settings
settings_search = Pesquisa
settings_browsing = Navegação
settings_address_bar = Barra de endereços
settings_privacy = Privacidade
search_engine = Mecanismo de busca
search_custom = Personalizado
search_custom_template = Endereço de pesquisa
search_custom_placeholder = https://example.com/search?q=%s
search_custom_caption = O endereço da página de resultados, com %s no lugar da pesquisa.
search_suggestions = Sugestões de pesquisa
search_suggestions_caption = Envia ao mecanismo de busca o que você digita na barra de endereços, enquanto você digita.
library_suggestions = Sugestões do Net Skip
library_suggestions_caption = Sugere páginas do seu histórico e dos seus favoritos enquanto você digita.
settings_tabs = Abas
address_at_top = Barra de endereços no topo
address_at_top_caption = Em um celular, mostra a barra de endereços acima da página em vez de abaixo.
open_links_background = Abrir links em segundo plano
close_tabs = Fechar abas
close_tabs_manually = Manualmente
close_tabs_day = Após um dia
close_tabs_week = Após uma semana
close_tabs_month = Após um mês
home_page = Página inicial
home_page_placeholder = Página de início
restore_tabs = Reabrir as abas da última vez
show_frequent = Mostrar sites visitados com frequência
full_address = Mostrar o endereço completo do site
upgrade_https = Atualizar para HTTPS
upgrade_https_caption = Carrega com segurança os endereços http:// digitados quando o site permite.
page_zoom = Zoom da página
blocking = Anúncios e rastreadores
blocking_level = Bloqueio
blocking_off = Desativado
blocking_light = Leve
blocking_strong = Forte
blocking_aggressive = Agressivo
blocking_caption = “Leve” bloqueia a maioria dos rastreadores e alguns anúncios. “Forte” bloqueia mais rastreadores e anúncios; alguns sites podem deixar de funcionar. “Agressivo” bloqueia quase todos os rastreadores e anúncios; sites vão deixar de funcionar.
blocking_update = Atualizar listas agora
blocking_unavailable = O bloqueio de conteúdo não está disponível.
blocking_status_off = O bloqueio está desativado.
blocking_status_waiting = Baixando listas de filtros…
blocking_status_ready = { $lists ->
    [one] Usando { $lists } lista de filtros, sempre atualizada.
   *[other] Usando { $lists } listas de filtros, sempre atualizadas.
}
blocking_status_failed = Não foi possível baixar as listas de filtros. O Net Skip tentará novamente.
blocking_custom = Personalizado
blocking_filter_lists = Listas de filtros
blocking_lists_caption = Ativar ou desativar uma lista, ou mudar uma opção, deixa o bloqueio como “Personalizado”. Restaure uma predefinição para recomeçar.
blocking_lists_general = Listas
blocking_lists_regional = Listas regionais
blocking_lists_added = Listas adicionadas
blocking_purpose_privacy = Rastreadores
blocking_purpose_ads = Anúncios
blocking_purpose_cookies = Avisos de cookies
blocking_purpose_annoyances = Elementos incômodos
blocking_purpose_social = Widgets sociais
blocking_purpose_regional = Anúncios regionais
blocking_purpose_security = Malware e golpes
blocking_list_not_downloaded = Não baixada
blocking_list_failed = Falha no download
blocking_list_stale = Falha na atualização; usando a última cópia
blocking_list_updated_now = Atualizada agora mesmo
blocking_list_updated_minutes = { $n ->
    [one] Atualizada há { $n } minuto
   *[other] Atualizada há { $n } minutos
}
blocking_list_updated_hours = { $n ->
    [one] Atualizada há { $n } hora
   *[other] Atualizada há { $n } horas
}
blocking_list_updated_days = { $n ->
    [one] Atualizada há { $n } dia
   *[other] Atualizada há { $n } dias
}
blocking_add_list = Adicionar lista
blocking_add_list_placeholder = https://example.com/list.txt
blocking_add_list_caption = Listas no formato Adblock Plus ou hosts funcionam. Regras que executam scripts ou reescrevem páginas são contadas, mas não aplicadas.
blocking_add_checking = Verificando a lista…
blocking_add_failed = Não foi possível adicionar esta lista: { $error }
blocking_add_done = { $rules ->
    [one] “{ $title }” adicionada: { $rules } regra.
   *[other] “{ $title }” adicionada: { $rules } regras.
}
blocking_add_unsupported = { $n ->
    [one] { $n } regra não pode ser aplicada.
   *[other] { $n } regras não podem ser aplicadas.
}
blocking_remove_list = Remover
blocking_options = Opções
blocking_document_blocking = Bloquear páginas de sites bloqueados
blocking_document_blocking_caption = Links para um site de uma lista não abrem.
blocking_same_site = Bloquear solicitações do próprio site
blocking_same_site_caption = Bloqueia anúncios que o próprio site exibe. Bloqueia mais, mas mais sites deixam de funcionar.
blocking_restore_preset = Restaurar predefinição
blocking_status_preparing = Preparando listas de filtros…
blocking_status_kept = Não foi possível baixar { $lists }. As listas anteriores continuam em uso.
blocking_reload_offer = As listas mudaram. Recarregue a página para usá-las.
blocking_reload_page = Recarregar página
blocking_this_site = Bloqueio neste site
blocking_site_default = Usar padrão ({ $level })
blocked_count = { $n ->
    [one] { $n } item bloqueado
   *[other] { $n } itens bloqueados
}
clear_history = Limpar histórico
clear_history_confirm = Limpar todo o histórico de navegação? Não é possível desfazer.
clear_website_data = Limpar dados de sites
clear_website_data_caption = Remove cookies, caches e dados salvos pelos sites, e desconecta você dos sites.
clear_website_data_confirm = Remover todos os cookies e dados de sites? Você será desconectado dos sites.
clear_bookmarks = Apagar todos os favoritos
clear_bookmarks_confirm = Apagar todos os favoritos? Não é possível desfazer.

## Permission prompts
# What the system shows when a web page asks for a device feature (Day.toml [permissions]).
permission_camera = O Net Skip precisa acessar sua câmera para que os sites visitados possam usá-la em chamadas de vídeo, leitura de códigos QR e captura de fotos quando você conceder permissão.
permission_microphone = O Net Skip precisa acessar seu microfone para que os sites visitados possam usá-lo em chamadas de voz e vídeo, busca por voz e gravação de áudio quando você conceder permissão.
permission_location_when_in_use = O Net Skip usa sua localização para oferecer recursos baseados em localização aos sites visitados, como mapas, resultados de buscas locais e informações do tempo quando você conceder permissão.
permission_motion = O Net Skip usa dados de movimento e orientação para que os sites visitados possam oferecer experiências interativas que respondem à forma como você move e inclina o dispositivo quando você conceder permissão.
permission_NSSpeechRecognitionUsageDescription = O Net Skip usa o reconhecimento de fala para que você possa ditar URLs, buscas e entradas de formulário por voz em vez de digitar.
