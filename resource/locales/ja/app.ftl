# Copyright © The Daybrite Project
# SPDX-License-Identifier: MPL-2.0
#
# All app-owned UI text. Settings and About labels come from Day's catalog.
app_title = Net Skip
about_tagline = じゃまにならないウェブブラウザ。

## Menus
menu_file = ファイル
menu_edit = 編集
menu_view = 表示
menu_history = 履歴
menu_bookmarks = ブックマーク
menu_tabs = タブ

## Tabs
new_tab = 新規タブ
new_private_tab = 新規プライベートタブ
private_tab = プライベートタブ
close_tab = タブを閉じる
close_other_tabs = 他のタブを閉じる
duplicate_tab = タブを複製
reopen_closed_tab = 閉じたタブを開き直す
recently_closed = 最近閉じたタブ
no_closed_tabs = 最近閉じたタブはありません
show_all_tabs = すべてのタブを表示
next_tab = 次のタブ
previous_tab = 前のタブ
tab_number = タブ { $n }
last_tab = 最後のタブ
tab_count = { $n ->
   *[other] { $n }個のタブ
}
open_in_new_tab = 新規タブで開く
open_in_background_tab = バックグラウンドのタブで開く
open_location = 場所を開く…
private_explainer = プライベートブラウズ：このタブで表示したページは履歴に残らず、タブを閉じるとそのタブのことも忘れます。

## Navigation
back = 戻る
forward = 進む
reload = ページを再読み込み
stop_loading = 読み込みを停止
home = ホーム
share = 共有
more = その他
done = 完了
edit = 編集
cancel = キャンセル
settings = 設定

## The address bar
address_placeholder = { $engine }で検索、またはアドレスを入力
secure_connection = 安全な接続
clear_text = クリア
search_with = { $engine }で検索
go_to_site = サイトへ移動
switch_to_tab = タブに切り替え · { $site }
fill_in_search = 検索欄に入力

## Find in page and zoom
find_on_page = ページ内を検索…
find_placeholder = ページ内を検索
find_previous = 前の一致
find_next = 次の一致
find_none = 見つかりません
find_count = { $current }/{ $total }
text_size = テキストサイズ
desktop_site = デスクトップ用サイト
request_desktop_site = デスクトップ用Webサイトを表示
request_mobile_site = モバイル用Webサイトを表示
hide_toolbar = ツールバーを非表示
actual_size_percent = 実際のサイズ（{ $percent }%）
page_menu = ページメニュー
show_toolbar = ツールバーを表示
zoom_in = 拡大
zoom_out = 縮小
actual_size = 実際のサイズ

## Bookmarks and history
show_library = ブックマークと履歴
show_bookmarks = ブックマークを表示
show_all_history = すべての履歴を表示
bookmarks = ブックマーク
history = 履歴
add_bookmark = ブックマークに追加
remove_bookmark = ブックマークを削除
copy_link = リンクをコピー
delete_history_item = 履歴から削除
search_bookmarks = ブックマークを検索
search_history = 履歴を検索
bookmarks_empty = ページをブックマークすると、こことスタートページに表示されます。
history_empty = 表示したページがここに表示されます。
no_matches = 検索に一致する項目はありません。
favorites = お気に入り
favorites_empty = ブックマークがここに表示されます。
frequently_visited = よく閲覧するサイト
today = 今日
yesterday = 昨日
monday = 月曜日
tuesday = 火曜日
wednesday = 水曜日
thursday = 木曜日
friday = 金曜日
saturday = 土曜日
sunday = 日曜日
clear_history_button = 消去…
clear_history_prompt = 消去する履歴の期間：
clear_last_hour = 過去1時間
clear_today = 今日
clear_two_days = 今日と昨日
clear_all_history = すべての履歴
clear_history_message = 消去すると、表示したページが履歴から削除されます。
delete = 削除

## Settings
settings_search = 検索
settings_browsing = ブラウズ
settings_address_bar = アドレスバー
settings_privacy = プライバシー
search_engine = 検索エンジン
search_custom = カスタム
search_custom_template = 検索アドレス
search_custom_placeholder = https://example.com/search?q=%s
search_custom_caption = 検索結果ページのアドレス。検索語が入る位置に %s を置きます。
search_suggestions = 検索候補
search_suggestions_caption = アドレスバーに入力した内容を、入力中に検索エンジンへ送信します。
library_suggestions = Net Skipの候補
library_suggestions_caption = 入力中に、履歴とブックマークからページを候補として表示します。
settings_tabs = タブ
address_at_top = アドレスバーを上部に表示
address_at_top_caption = スマートフォンで、アドレスバーをページの下ではなく上に表示します。
open_links_background = リンクをバックグラウンドで開く
close_tabs = タブを閉じる
close_tabs_manually = 手動
close_tabs_day = 1日後
close_tabs_week = 1週間後
close_tabs_month = 1か月後
home_page = ホームページ
home_page_placeholder = スタートページ
restore_tabs = 前回のタブを開き直す
show_frequent = よく閲覧するサイトを表示
full_address = Webサイトの完全なアドレスを表示
upgrade_https = HTTPSにアップグレード
upgrade_https_caption = 入力した http:// のアドレスを、サイトが対応していれば安全に読み込みます。
page_zoom = ページのズーム
blocking = 広告とトラッカー
blocking_level = ブロック
blocking_off = オフ
blocking_light = 弱
blocking_strong = 強
blocking_aggressive = 最強
blocking_caption = 「弱」はほとんどのトラッカーと一部の広告をブロックします。「強」はより多くのトラッカーと広告をブロックしますが、一部のサイトが正しく動作しなくなることがあります。「最強」はほぼすべてのトラッカーと広告をブロックしますが、サイトが正しく動作しなくなります。
blocking_update = 今すぐリストを更新
blocking_unavailable = コンテンツブロックは利用できません。
blocking_status_off = ブロックはオフです。
blocking_status_waiting = フィルタリストをダウンロード中…
blocking_status_ready = { $lists ->
   *[other] { $lists }個のフィルタリストを使用中です。リストは常に最新の状態に保たれます。
}
blocking_status_failed = フィルタリストをダウンロードできませんでした。Net Skipが後でもう一度試します。
blocking_custom = カスタム
blocking_filter_lists = フィルタリスト
blocking_lists_caption = リストのオン／オフを切り替えたりオプションを変更したりすると、ブロックは「カスタム」になります。最初からやり直すには、プリセットを復元してください。
blocking_lists_general = リスト
blocking_lists_regional = 地域別リスト
blocking_lists_added = 追加したリスト
blocking_purpose_privacy = トラッカー
blocking_purpose_ads = 広告
blocking_purpose_cookies = Cookieの通知
blocking_purpose_annoyances = 迷惑な要素
blocking_purpose_social = ソーシャルウィジェット
blocking_purpose_regional = 地域別の広告
blocking_purpose_security = マルウェアと詐欺
blocking_list_not_downloaded = 未ダウンロード
blocking_list_failed = ダウンロードに失敗しました
blocking_list_stale = 更新に失敗したため、前回のコピーを使用中
blocking_list_updated_now = たった今更新
blocking_list_updated_minutes = { $n ->
   *[other] { $n }分前に更新
}
blocking_list_updated_hours = { $n ->
   *[other] { $n }時間前に更新
}
blocking_list_updated_days = { $n ->
   *[other] { $n }日前に更新
}
blocking_add_list = リストを追加
blocking_add_list_placeholder = https://example.com/list.txt
blocking_add_list_caption = Adblock Plus形式またはhosts形式のリストを使用できます。スクリプトを実行するルールやページを書き換えるルールは、数には含まれますが適用されません。
blocking_add_checking = リストを確認中…
blocking_add_failed = このリストを追加できませんでした：{ $error }
blocking_add_done = { $rules ->
   *[other] 「{ $title }」を追加しました：{ $rules }件のルール
}
blocking_add_unsupported = { $n ->
   *[other] { $n }件のルールは適用できません。
}
blocking_remove_list = 削除
blocking_options = オプション
blocking_document_blocking = ブロック対象サイトのページをブロック
blocking_document_blocking_caption = リストに含まれるサイトへのリンクは開きません。
blocking_same_site = サイト自身のリクエストをブロック
blocking_same_site_caption = サイト自身が配信する広告をブロックします。より多くブロックできますが、正しく動作しないサイトが増えます。
blocking_restore_preset = プリセットを復元
blocking_status_preparing = フィルタリストを準備中…
blocking_status_kept = { $lists }をダウンロードできませんでした。以前のリストを引き続き使用します。
blocking_reload_offer = リストが変更されました。適用するにはページを再読み込みしてください。
blocking_reload_page = ページを再読み込み
blocking_this_site = このサイトでのブロック
blocking_site_default = デフォルトを使用（{ $level }）
blocked_count = { $n ->
   *[other] { $n }件をブロックしました
}
clear_history = 履歴を消去
clear_history_confirm = すべての閲覧履歴を消去しますか？この操作は取り消せません。
clear_website_data = Webサイトデータを消去
clear_website_data_caption = Cookie、キャッシュ、保存されたサイトデータを削除します。Webサイトからはサインアウトされます。
clear_website_data_confirm = すべてのCookieとWebサイトデータを削除しますか？Webサイトからサインアウトされます。
clear_bookmarks = すべてのブックマークを削除
clear_bookmarks_confirm = すべてのブックマークを削除しますか？この操作は取り消せません。

## Downloads, links to other apps, sign-in and error pages
downloads = ダウンロード
download_confirm = “{ $name }”をダウンロードしますか？
download = ダウンロード
search_downloads = ダウンロードを検索
downloads_empty = ダウンロードしたファイルがここに表示されます。
download_progress = { $received } / { $total }
download_failed = 失敗
download_canceled = キャンセル済み
clear_downloads = 消去
remove_download = リストから削除
open_app_title = ほかのアプリで開きますか？
open_app_message = このリンクはNet Skipの外で開きます: { $url }
open = 開く
sign_in_title = サインイン
sign_in_message = “{ $host }”が名前とパスワードを求めています。
sign_in_failed = その名前またはパスワードは受け付けられませんでした。
user_name = ユーザ名
password = パスワード
sign_in = サインイン
error_offline_title = オフラインです
error_offline_message = インターネットに接続してから、やり直してください。
error_host_title = サイトが見つかりません
error_host_message = Net Skipはサーバ“{ $host }”を見つけられません。
error_timeout_title = サイトの応答に時間がかかりすぎています
error_timeout_message = “{ $host }”は時間内に応答しませんでした。
error_connection_title = 接続できません
error_connection_message = Net Skipは“{ $host }”に接続できませんでした。
error_secure_title = 安全に接続できません
error_secure_message = Net Skipは“{ $host }”への安全な接続を確立できませんでした。
error_certificate_title = この接続はプライベートではありません
error_certificate_message = “{ $host }”のセキュリティ証明書は信頼されていません。誰かがこのサイトになりすまして、あなたの情報を盗もうとしている可能性があります。
error_crashed_title = このページは動作を停止しました
error_crashed_message = ページに問題が起きたため、閉じられました。
error_other_title = このページを開けません
error_other_message = Net Skipは“{ $host }”のページを開けませんでした。
try_again = やり直す
visit_anyway = このWebサイトを閲覧
visit_anyway_title = このWebサイトを閲覧しますか？
visit_anyway_message = “{ $host }”への接続はプライベートではなくなります。

## The user agent setting
settings_websites = Webサイト
user_agent = ユーザエージェント
user_agent_caption = Net SkipがWebサイトに伝える自身の種類です。変更するとページが再読み込みされます。
user_agent_default = デフォルト
user_agent_engine = 組み込みWebビュー
user_agent_custom = カスタムユーザエージェント

## Permission prompts
# What the system shows when a web page asks for a device feature (Day.toml [permissions]).
permission_camera = Net Skipは、許可した場合に閲覧中のウェブサイトがビデオ通話、QRコードのスキャン、写真撮影に利用できるよう、カメラへのアクセスを必要とします。
permission_microphone = Net Skipは、許可した場合に閲覧中のウェブサイトが音声・ビデオ通話、音声検索、音声録音に利用できるよう、マイクへのアクセスを必要とします。
permission_location_when_in_use = Net Skipは、許可した場合に閲覧中のウェブサイトに地図、近隣の検索結果、天気情報などの位置情報機能を提供するために位置情報を使用します。
permission_motion = Net Skipは、許可した場合に閲覧中のウェブサイトがデバイスの動きや傾きに反応するインタラクティブな体験を提供できるよう、モーションおよび方向データを使用します。
permission_NSSpeechRecognitionUsageDescription = Net Skipは、URLや検索クエリ、フォームへの入力をタイピングの代わりに音声で行えるよう、音声認識を使用します。
