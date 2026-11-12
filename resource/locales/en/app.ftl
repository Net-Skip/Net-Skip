# Copyright © The Daybrite Project
# SPDX-License-Identifier: MPL-2.0
#
# All app-owned UI text. Settings and About labels come from Day's catalog.
app_title = Net Skip
about_tagline = A web browser that keeps out of the way.

## Menus
menu_file = File
menu_edit = Edit
menu_view = View
menu_history = History
menu_bookmarks = Bookmarks
menu_tabs = Tabs

## Tabs
new_tab = New Tab
new_private_tab = New Private Tab
private_tab = Private Tab
close_tab = Close Tab
close_other_tabs = Close Other Tabs
duplicate_tab = Duplicate Tab
reopen_closed_tab = Reopen Closed Tab
recently_closed = Recently Closed
no_closed_tabs = No Recently Closed Tabs
show_all_tabs = Show All Tabs
next_tab = Next Tab
previous_tab = Previous Tab
tab_number = Tab { $n }
last_tab = Last Tab
tab_count = { $n ->
    [one] 1 Tab
   *[other] { $n } Tabs
}
open_in_new_tab = Open in New Tab
open_in_background_tab = Open in Background Tab
open_location = Open Location…
private_explainer = Private browsing: pages you visit in this tab stay out of your history, and the tab is forgotten when you close it.

## Navigation
back = Back
forward = Forward
reload = Reload Page
stop_loading = Stop Loading
home = Home
share = Share
more = More
done = Done
edit = Edit
cancel = Cancel
settings = Settings

## The address bar
address_placeholder = Search { $engine } or enter address
secure_connection = Secure connection
clear_text = Clear
search_with = Search { $engine }
go_to_site = Go to site
switch_to_tab = Switch to tab · { $site }
fill_in_search = Fill in search

## Find in page and zoom
find_on_page = Find on Page…
find_placeholder = Find on page
find_previous = Previous Match
find_next = Next Match
find_none = Not found
find_count = { $current } of { $total }
text_size = Text Size
desktop_site = Desktop site
request_desktop_site = Request Desktop Website
request_mobile_site = Request Mobile Website
hide_toolbar = Hide Toolbar
actual_size_percent = Actual Size ({ $percent }%)
page_menu = Page Menu
show_toolbar = Show Toolbar
zoom_in = Zoom In
zoom_out = Zoom Out
actual_size = Actual Size

## Bookmarks and history
show_library = Bookmarks and History
show_bookmarks = Show Bookmarks
show_all_history = Show All History
bookmarks = Bookmarks
history = History
add_bookmark = Add Bookmark
remove_bookmark = Remove Bookmark
copy_link = Copy Link
delete_history_item = Remove from History
search_bookmarks = Search bookmarks
search_history = Search history
bookmarks_empty = Bookmark a page to keep it here and on the start page.
history_empty = Pages you visit appear here.
no_matches = Nothing matches your search.
favorites = Favorites
favorites_empty = Your bookmarks appear here.
frequently_visited = Frequently Visited
today = Today
yesterday = Yesterday
monday = Monday
tuesday = Tuesday
wednesday = Wednesday
thursday = Thursday
friday = Friday
saturday = Saturday
sunday = Sunday
clear_history_button = Clear…
clear_history_prompt = Clear history from:
clear_last_hour = The Last Hour
clear_today = Today
clear_two_days = Today and Yesterday
clear_all_history = All History
clear_history_message = Clearing removes the pages you visited from your history.
delete = Delete

## Settings
settings_search = Search
settings_browsing = Browsing
settings_address_bar = Address Bar
settings_privacy = Privacy
search_engine = Search engine
search_custom = Custom
search_custom_template = Search address
search_custom_placeholder = https://example.com/search?q=%s
search_custom_caption = The results page's address, with %s where the search goes.
search_suggestions = Search suggestions
search_suggestions_caption = Sends what you type in the address bar to the search engine as you type.
library_suggestions = Net Skip suggestions
library_suggestions_caption = Offers pages from your history and bookmarks as you type.
settings_tabs = Tabs
address_at_top = Address bar at top
address_at_top_caption = On a phone, shows the address bar above the page instead of below it.
open_links_background = Open links in background
close_tabs = Close tabs
close_tabs_manually = Manually
close_tabs_day = After One Day
close_tabs_week = After One Week
close_tabs_month = After One Month
home_page = Home page
home_page_placeholder = Start page
restore_tabs = Reopen tabs from last time
show_frequent = Show frequently visited sites
full_address = Show full website address
upgrade_https = Upgrade to HTTPS
upgrade_https_caption = Loads typed http:// addresses securely when the site allows it.
page_zoom = Page zoom
blocking = Ads and Trackers
blocking_level = Blocking
blocking_off = Off
blocking_light = Light
blocking_strong = Strong
blocking_aggressive = Aggressive
blocking_caption = Light blocks most trackers and some ads. Strong blocks more trackers and ads; some sites might break. Aggressive blocks almost all trackers and ads; sites will break.
blocking_update = Update Lists Now
blocking_unavailable = Content blocking isn't available.
blocking_status_off = Blocking is off.
blocking_status_waiting = Downloading filter lists…
blocking_status_ready = { $lists ->
    [one] Using 1 filter list, kept up to date.
   *[other] Using { $lists } filter lists, kept up to date.
}
blocking_status_failed = Couldn't download the filter lists. Net Skip will try again.
blocking_custom = Custom
blocking_filter_lists = Filter Lists
blocking_lists_caption = Switching a list on or off, or changing an option, makes blocking Custom. Restore a preset to start over.
blocking_lists_general = Lists
blocking_lists_regional = Regional Lists
blocking_lists_added = Added Lists
blocking_purpose_privacy = Trackers
blocking_purpose_ads = Ads
blocking_purpose_cookies = Cookie notices
blocking_purpose_annoyances = Annoyances
blocking_purpose_social = Social widgets
blocking_purpose_regional = Regional ads
blocking_purpose_security = Malware and scams
blocking_list_not_downloaded = Not downloaded
blocking_list_failed = Download failed
blocking_list_stale = Update failed; using the last copy
blocking_list_updated_now = Updated just now
blocking_list_updated_minutes = { $n ->
    [one] Updated 1 minute ago
   *[other] Updated { $n } minutes ago
}
blocking_list_updated_hours = { $n ->
    [one] Updated 1 hour ago
   *[other] Updated { $n } hours ago
}
blocking_list_updated_days = { $n ->
    [one] Updated 1 day ago
   *[other] Updated { $n } days ago
}
blocking_add_list = Add List
blocking_add_list_placeholder = https://example.com/list.txt
blocking_add_list_caption = Lists in Adblock Plus or hosts format work. Rules that run scripts or rewrite pages are counted but not applied.
blocking_add_checking = Checking the list…
blocking_add_failed = Couldn't add this list: { $error }
blocking_add_done = { $rules ->
    [one] Added “{ $title }”: 1 rule.
   *[other] Added “{ $title }”: { $rules } rules.
}
blocking_add_unsupported = { $n ->
    [one] 1 rule can't be applied.
   *[other] { $n } rules can't be applied.
}
blocking_remove_list = Remove
blocking_options = Options
blocking_document_blocking = Block pages on blocked sites
blocking_document_blocking_caption = Links to a site on a list don't open.
blocking_same_site = Block the site's own requests
blocking_same_site_caption = Blocks ads a site serves itself. Catches more, but more sites break.
blocking_restore_preset = Restore Preset
blocking_status_preparing = Preparing filter lists…
blocking_status_kept = Couldn't download { $lists }. The previous lists stay in use.
blocking_reload_offer = The lists changed. Reload the page to use them.
blocking_reload_page = Reload Page
blocking_this_site = Blocking on This Site
blocking_site_default = Use Default ({ $level })
blocked_count = { $n ->
    [one] 1 item blocked
   *[other] { $n } items blocked
}
clear_history = Clear History
clear_history_confirm = Clear all browsing history? This can't be undone.
clear_website_data = Clear Website Data
clear_website_data_caption = Removes cookies, caches and saved site data, signing you out of websites.
clear_website_data_confirm = Remove all cookies and website data? You will be signed out of websites.
clear_bookmarks = Delete All Bookmarks
clear_bookmarks_confirm = Delete every bookmark? This can't be undone.

## Downloads, links to other apps, sign-in and error pages
downloads = Downloads
download_confirm = Download “{ $name }”?
download = Download
search_downloads = Search downloads
downloads_empty = Files you download appear here.
download_progress = { $received } of { $total }
download_failed = Failed
download_canceled = Canceled
clear_downloads = Clear
remove_download = Remove from List
open_app_title = Open in another app?
open_app_message = This link opens outside Net Skip: { $url }
open = Open
sign_in_title = Sign In
sign_in_message = “{ $host }” asks for a name and password.
sign_in_failed = That name or password was not accepted.
user_name = User Name
password = Password
sign_in = Sign In
error_offline_title = You’re Offline
error_offline_message = Connect to the internet, then try again.
error_host_title = Site Not Found
error_host_message = Net Skip can’t find the server “{ $host }”.
error_timeout_title = The Site Took Too Long to Respond
error_timeout_message = “{ $host }” didn’t answer in time.
error_connection_title = Can’t Connect
error_connection_message = Net Skip couldn’t connect to “{ $host }”.
error_secure_title = Can’t Connect Securely
error_secure_message = Net Skip couldn’t make a secure connection to “{ $host }”.
error_certificate_title = This Connection Is Not Private
error_certificate_message = “{ $host }” has a security certificate that is not trusted. Someone may be impersonating the site to steal your information.
error_crashed_title = This Page Stopped Working
error_crashed_message = A problem with the page made it close.
error_other_title = This Page Can’t Be Opened
error_other_message = Net Skip couldn’t open the page at “{ $host }”.
try_again = Try Again
visit_anyway = Visit Website Anyway
visit_anyway_title = Visit this website?
visit_anyway_message = Your connection to “{ $host }” will not be private.

## The user agent setting
settings_websites = Websites
user_agent = User Agent
user_agent_caption = What Net Skip tells websites it is. Pages load again when this changes.
user_agent_default = Default
user_agent_engine = Embedded Web View
user_agent_custom = Custom User Agent

## Permission prompts
# What the system shows when a web page asks for a device feature (Day.toml [permissions]).
permission_camera = Net Skip needs access to your camera so that websites you visit can use it for video calls, scanning QR codes, and capturing photos when you grant them permission.
permission_microphone = Net Skip needs access to your microphone so that websites you visit can use it for voice and video calls, voice search, and recording audio when you grant them permission.
permission_location_when_in_use = Net Skip uses your location to provide location-aware features to websites you visit, such as maps, local search results, and weather information when you grant them permission.
permission_motion = Net Skip uses motion and orientation data so that websites you visit can offer interactive experiences that respond to how you move and tilt your device when you grant them permission.
permission_NSSpeechRecognitionUsageDescription = Net Skip uses speech recognition so that you can dictate URLs, search queries, and form input by voice instead of typing.
