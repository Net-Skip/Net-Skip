# Copyright © The Daybrite Project
# SPDX-License-Identifier: MPL-2.0
#
# All app-owned UI text. Settings and About labels come from Day's catalog.
app_title = Net Skip
about_tagline = Peramban web yang tidak menghalangi.

## Menus
menu_file = File
menu_edit = Edit
menu_view = Tampilan
menu_history = Riwayat
menu_bookmarks = Bookmark
menu_tabs = Tab

## Tabs
new_tab = Tab Baru
new_private_tab = Tab Pribadi Baru
private_tab = Tab Pribadi
close_tab = Tutup Tab
close_other_tabs = Tutup Tab Lain
duplicate_tab = Duplikat Tab
reopen_closed_tab = Buka Lagi Tab Tertutup
recently_closed = Baru Ditutup
no_closed_tabs = Tidak Ada Tab yang Baru Ditutup
show_all_tabs = Tampilkan Semua Tab
next_tab = Tab Berikutnya
previous_tab = Tab Sebelumnya
tab_number = Tab { $n }
last_tab = Tab Terakhir
tab_count = { $n ->
   *[other] { $n } Tab
}
open_in_new_tab = Buka di Tab Baru
open_in_background_tab = Buka di Tab Latar Belakang
open_location = Buka Alamat…
private_explainer = Penjelajahan pribadi: halaman yang Anda kunjungi di tab ini tidak masuk ke riwayat, dan tab ini dilupakan saat Anda menutupnya.

## Navigation
back = Kembali
forward = Maju
reload = Muat Ulang Halaman
stop_loading = Hentikan Pemuatan
home = Beranda
share = Bagikan
more = Lainnya
done = Selesai
edit = Edit
cancel = Batal
settings = Pengaturan

## The address bar
address_placeholder = Cari di { $engine } atau masukkan alamat
secure_connection = Koneksi aman
clear_text = Hapus
search_with = Cari di { $engine }
go_to_site = Buka situs
switch_to_tab = Beralih ke tab · { $site }
fill_in_search = Masukkan ke pencarian

## Find in page and zoom
find_on_page = Cari di Halaman…
find_placeholder = Cari di halaman
find_previous = Hasil Sebelumnya
find_next = Hasil Berikutnya
find_none = Tidak ditemukan
find_count = { $current } dari { $total }
text_size = Ukuran Teks
desktop_site = Situs desktop
request_desktop_site = Minta Situs Web Desktop
request_mobile_site = Minta Situs Web Seluler
hide_toolbar = Sembunyikan Bilah Alat
actual_size_percent = Ukuran Sebenarnya ({ $percent }%)
page_menu = Menu Halaman
show_toolbar = Tampilkan Bilah Alat
zoom_in = Perbesar
zoom_out = Perkecil
actual_size = Ukuran Sebenarnya

## Bookmarks and history
show_library = Bookmark dan Riwayat
show_bookmarks = Tampilkan Bookmark
show_all_history = Tampilkan Semua Riwayat
bookmarks = Bookmark
history = Riwayat
add_bookmark = Tambah Bookmark
remove_bookmark = Hapus Bookmark
copy_link = Salin Tautan
delete_history_item = Hapus dari Riwayat
search_bookmarks = Cari bookmark
search_history = Cari riwayat
bookmarks_empty = Bookmark sebuah halaman untuk menyimpannya di sini dan di halaman awal.
history_empty = Halaman yang Anda kunjungi muncul di sini.
no_matches = Tidak ada yang cocok dengan pencarian Anda.
favorites = Favorit
favorites_empty = Bookmark Anda muncul di sini.
frequently_visited = Sering Dikunjungi
today = Hari Ini
yesterday = Kemarin
monday = Senin
tuesday = Selasa
wednesday = Rabu
thursday = Kamis
friday = Jumat
saturday = Sabtu
sunday = Minggu
clear_history_button = Hapus…
clear_history_prompt = Hapus riwayat dari:
clear_last_hour = Satu Jam Terakhir
clear_today = Hari Ini
clear_two_days = Hari Ini dan Kemarin
clear_all_history = Semua Riwayat
clear_history_message = Penghapusan akan membuang halaman yang Anda kunjungi dari riwayat.
delete = Hapus

## Settings
settings_search = Pencarian
settings_browsing = Penjelajahan
settings_address_bar = Bilah Alamat
settings_privacy = Privasi
search_engine = Mesin pencari
search_custom = Khusus
search_custom_template = Alamat pencarian
search_custom_placeholder = https://example.com/search?q=%s
search_custom_caption = Alamat halaman hasil, dengan %s di tempat kata yang dicari.
search_suggestions = Saran pencarian
search_suggestions_caption = Mengirim apa yang Anda ketik di bilah alamat ke mesin pencari selagi Anda mengetik.
library_suggestions = Saran Net Skip
library_suggestions_caption = Menawarkan halaman dari riwayat dan bookmark Anda selagi Anda mengetik.
settings_tabs = Tab
address_at_top = Bilah alamat di atas
address_at_top_caption = Di ponsel, menampilkan bilah alamat di atas halaman, bukan di bawahnya.
open_links_background = Buka tautan di latar belakang
close_tabs = Tutup tab
close_tabs_manually = Secara Manual
close_tabs_day = Setelah Satu Hari
close_tabs_week = Setelah Satu Minggu
close_tabs_month = Setelah Satu Bulan
home_page = Halaman beranda
home_page_placeholder = Halaman awal
restore_tabs = Buka lagi tab dari terakhir kali
show_frequent = Tampilkan situs yang sering dikunjungi
full_address = Tampilkan alamat situs web lengkap
upgrade_https = Tingkatkan ke HTTPS
upgrade_https_caption = Memuat alamat http:// yang diketik secara aman jika situs mengizinkannya.
page_zoom = Zoom halaman
blocking = Iklan dan Pelacak
blocking_level = Pemblokiran
blocking_off = Mati
blocking_light = Ringan
blocking_strong = Kuat
blocking_aggressive = Agresif
blocking_caption = “Ringan” memblokir sebagian besar pelacak dan beberapa iklan. “Kuat” memblokir lebih banyak pelacak dan iklan; beberapa situs mungkin tidak berfungsi. “Agresif” memblokir hampir semua pelacak dan iklan; situs akan bermasalah.
blocking_update = Perbarui Daftar Sekarang
blocking_unavailable = Pemblokiran konten tidak tersedia.
blocking_status_off = Pemblokiran nonaktif.
blocking_status_waiting = Mengunduh daftar filter…
blocking_status_ready = { $lists ->
   *[other] Menggunakan { $lists } daftar filter yang selalu diperbarui.
}
blocking_status_failed = Tidak dapat mengunduh daftar filter. Net Skip akan mencoba lagi.
blocking_custom = Kustom
blocking_filter_lists = Daftar Filter
blocking_lists_caption = Mengaktifkan atau menonaktifkan daftar, atau mengubah opsi, membuat pemblokiran menjadi “Kustom”. Pulihkan prasetel untuk memulai lagi.
blocking_lists_general = Daftar
blocking_lists_regional = Daftar Regional
blocking_lists_added = Daftar yang Ditambahkan
blocking_purpose_privacy = Pelacak
blocking_purpose_ads = Iklan
blocking_purpose_cookies = Pemberitahuan cookie
blocking_purpose_annoyances = Gangguan
blocking_purpose_social = Widget sosial
blocking_purpose_regional = Iklan regional
blocking_purpose_security = Malware dan penipuan
blocking_list_not_downloaded = Belum diunduh
blocking_list_failed = Gagal mengunduh
blocking_list_stale = Gagal memperbarui; memakai salinan terakhir
blocking_list_updated_now = Baru saja diperbarui
blocking_list_updated_minutes = { $n ->
   *[other] Diperbarui { $n } menit lalu
}
blocking_list_updated_hours = { $n ->
   *[other] Diperbarui { $n } jam lalu
}
blocking_list_updated_days = { $n ->
   *[other] Diperbarui { $n } hari lalu
}
blocking_add_list = Tambah Daftar
blocking_add_list_placeholder = https://example.com/list.txt
blocking_add_list_caption = Daftar dalam format Adblock Plus atau hosts dapat digunakan. Aturan yang menjalankan skrip atau menulis ulang halaman dihitung, tetapi tidak diterapkan.
blocking_add_checking = Memeriksa daftar…
blocking_add_failed = Tidak dapat menambahkan daftar ini: { $error }
blocking_add_done = { $rules ->
   *[other] “{ $title }” ditambahkan: { $rules } aturan.
}
blocking_add_unsupported = { $n ->
   *[other] { $n } aturan tidak dapat diterapkan.
}
blocking_remove_list = Hapus
blocking_options = Opsi
blocking_document_blocking = Blokir halaman di situs yang diblokir
blocking_document_blocking_caption = Tautan ke situs dalam daftar tidak akan terbuka.
blocking_same_site = Blokir permintaan situs itu sendiri
blocking_same_site_caption = Memblokir iklan yang disajikan situs itu sendiri. Lebih banyak yang terblokir, tetapi lebih banyak situs bermasalah.
blocking_restore_preset = Pulihkan Prasetel
blocking_status_preparing = Menyiapkan daftar filter…
blocking_status_kept = Tidak dapat mengunduh { $lists }. Daftar sebelumnya tetap digunakan.
blocking_reload_offer = Daftar telah berubah. Muat ulang halaman untuk menggunakannya.
blocking_reload_page = Muat Ulang Halaman
blocking_this_site = Pemblokiran di Situs Ini
blocking_site_default = Gunakan Default ({ $level })
blocked_count = { $n ->
   *[other] { $n } item diblokir
}
clear_history = Hapus Riwayat
clear_history_confirm = Hapus semua riwayat penjelajahan? Tindakan ini tidak dapat dibatalkan.
clear_website_data = Hapus Data Situs Web
clear_website_data_caption = Menghapus cookie, cache, dan data situs yang tersimpan, sehingga Anda keluar dari situs web.
clear_website_data_confirm = Hapus semua cookie dan data situs web? Anda akan keluar dari situs web.
clear_bookmarks = Hapus Semua Bookmark
clear_bookmarks_confirm = Hapus semua bookmark? Tindakan ini tidak dapat dibatalkan.

## Permission prompts
# What the system shows when a web page asks for a device feature (Day.toml [permissions]).
permission_camera = Net Skip memerlukan akses ke kamera Anda agar situs web yang Anda kunjungi dapat menggunakannya untuk panggilan video, memindai kode QR, dan mengambil foto saat Anda memberinya izin.
permission_microphone = Net Skip memerlukan akses ke mikrofon Anda agar situs web yang Anda kunjungi dapat menggunakannya untuk panggilan suara dan video, pencarian suara, serta merekam audio saat Anda memberinya izin.
permission_location_when_in_use = Net Skip menggunakan lokasi Anda untuk menyediakan fitur berbasis lokasi pada situs web yang Anda kunjungi, seperti peta, hasil pencarian lokal, dan informasi cuaca saat Anda memberinya izin.
permission_motion = Net Skip menggunakan data gerak dan orientasi agar situs web yang Anda kunjungi dapat menyajikan pengalaman interaktif yang merespons cara Anda menggerakkan dan memiringkan perangkat saat Anda memberinya izin.
permission_NSSpeechRecognitionUsageDescription = Net Skip menggunakan pengenalan suara agar Anda dapat mendiktekan URL, kueri pencarian, dan masukan formulir lewat suara tanpa perlu mengetik.
