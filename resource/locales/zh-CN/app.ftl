# Copyright © The Daybrite Project
# SPDX-License-Identifier: MPL-2.0
#
# All app-owned UI text. Settings and About labels come from Day's catalog.
app_title = Net Skip
about_tagline = 不打扰你的网页浏览器。

## Menus
menu_file = 文件
menu_edit = 编辑
menu_view = 显示
menu_history = 历史记录
menu_bookmarks = 书签
menu_tabs = 标签页

## Tabs
new_tab = 新标签页
new_private_tab = 新无痕标签页
private_tab = 无痕标签页
close_tab = 关闭标签页
close_other_tabs = 关闭其他标签页
duplicate_tab = 复制标签页
reopen_closed_tab = 重新打开已关闭的标签页
recently_closed = 最近关闭
no_closed_tabs = 没有最近关闭的标签页
show_all_tabs = 显示所有标签页
next_tab = 下一个标签页
previous_tab = 上一个标签页
tab_number = 标签页 { $n }
last_tab = 最后一个标签页
tab_count = { $n ->
   *[other] { $n } 个标签页
}
open_in_new_tab = 在新标签页中打开
open_in_background_tab = 在后台标签页中打开
open_location = 打开网址…
private_explainer = 无痕浏览：你在此标签页中访问的页面不会记入历史记录，关闭此标签页后也不会保留它。

## Navigation
back = 返回
forward = 前进
reload = 重新加载页面
stop_loading = 停止加载
home = 主页
share = 分享
more = 更多
done = 完成
edit = 编辑
cancel = 取消
settings = 设置

## The address bar
address_placeholder = 使用 { $engine } 搜索或输入网址
secure_connection = 安全连接
clear_text = 清除
search_with = 使用 { $engine } 搜索
go_to_site = 前往网站
switch_to_tab = 切换到标签页 · { $site }
fill_in_search = 填入搜索栏

## Find in page and zoom
find_on_page = 在页面中查找…
find_placeholder = 在页面中查找
find_previous = 上一个匹配
find_next = 下一个匹配
find_none = 未找到
find_count = { $current }/{ $total }
text_size = 文字大小
desktop_site = 桌面版网站
request_desktop_site = 请求桌面网站
request_mobile_site = 请求移动网站
hide_toolbar = 隐藏工具栏
actual_size_percent = 实际大小（{ $percent }%）
page_menu = 页面菜单
show_toolbar = 显示工具栏
zoom_in = 放大
zoom_out = 缩小
actual_size = 实际大小

## Bookmarks and history
show_library = 书签和历史记录
show_bookmarks = 显示书签
show_all_history = 显示全部历史记录
bookmarks = 书签
history = 历史记录
add_bookmark = 添加书签
remove_bookmark = 移除书签
copy_link = 复制链接
delete_history_item = 从历史记录中移除
search_bookmarks = 搜索书签
search_history = 搜索历史记录
bookmarks_empty = 为页面添加书签后，就能在这里和起始页上找到它。
history_empty = 你访问过的页面会显示在这里。
no_matches = 没有与搜索匹配的内容。
favorites = 收藏夹
favorites_empty = 你的书签会显示在这里。
frequently_visited = 常去网站
today = 今天
yesterday = 昨天
monday = 星期一
tuesday = 星期二
wednesday = 星期三
thursday = 星期四
friday = 星期五
saturday = 星期六
sunday = 星期日
clear_history_button = 清除…
clear_history_prompt = 清除以下时间段的历史记录：
clear_last_hour = 最近一小时
clear_today = 今天
clear_two_days = 今天和昨天
clear_all_history = 全部历史记录
clear_history_message = 清除后，你访问过的页面将从历史记录中移除。
delete = 删除

## Settings
settings_search = 搜索
settings_browsing = 浏览
settings_address_bar = 地址栏
settings_privacy = 隐私
search_engine = 搜索引擎
search_custom = 自定
search_custom_template = 搜索网址
search_custom_placeholder = https://example.com/search?q=%s
search_custom_caption = 结果页面的网址，用 %s 标出搜索内容所在的位置。
search_suggestions = 搜索建议
search_suggestions_caption = 在你于地址栏中输入时，将输入的内容发送给搜索引擎。
library_suggestions = Net Skip 建议
library_suggestions_caption = 在你输入时，推荐历史记录和书签中的页面。
settings_tabs = 标签页
address_at_top = 地址栏位于顶部
address_at_top_caption = 在手机上，将地址栏显示在页面上方而不是下方。
open_links_background = 在后台打开链接
close_tabs = 关闭标签页
close_tabs_manually = 手动
close_tabs_day = 一天后
close_tabs_week = 一周后
close_tabs_month = 一个月后
home_page = 主页
home_page_placeholder = 起始页
restore_tabs = 重新打开上次的标签页
show_frequent = 显示常去网站
full_address = 显示完整网址
upgrade_https = 升级到 HTTPS
upgrade_https_caption = 网站允许时，以安全方式载入你输入的 http:// 网址。
page_zoom = 页面缩放
blocking = 广告和跟踪器
blocking_level = 拦截
blocking_off = 关闭
blocking_light = 轻度
blocking_strong = 强力
blocking_aggressive = 激进
blocking_caption = “轻度”会拦截大多数跟踪器和部分广告。“强力”会拦截更多跟踪器和广告，部分网站可能无法正常运行。“激进”会拦截几乎所有跟踪器和广告，网站将无法正常运行。
blocking_update = 立即更新列表
blocking_unavailable = 内容拦截不可用。
blocking_status_off = 拦截已关闭。
blocking_status_waiting = 正在下载过滤列表…
blocking_status_ready = { $lists ->
   *[other] 正在使用 { $lists } 个过滤列表，并保持最新。
}
blocking_status_failed = 无法下载过滤列表。Net Skip 稍后会重试。
blocking_custom = 自定义
blocking_filter_lists = 过滤列表
blocking_lists_caption = 开启或关闭列表，或更改选项，都会将拦截设为“自定义”。恢复预设即可重新开始。
blocking_lists_general = 列表
blocking_lists_regional = 地区列表
blocking_lists_added = 已添加的列表
blocking_purpose_privacy = 跟踪器
blocking_purpose_ads = 广告
blocking_purpose_cookies = Cookie 提示
blocking_purpose_annoyances = 干扰元素
blocking_purpose_social = 社交小组件
blocking_purpose_regional = 地区广告
blocking_purpose_security = 恶意软件和诈骗
blocking_list_not_downloaded = 未下载
blocking_list_failed = 下载失败
blocking_list_stale = 更新失败；正在使用上次的副本
blocking_list_updated_now = 刚刚更新
blocking_list_updated_minutes = { $n ->
   *[other] { $n } 分钟前更新
}
blocking_list_updated_hours = { $n ->
   *[other] { $n } 小时前更新
}
blocking_list_updated_days = { $n ->
   *[other] { $n } 天前更新
}
blocking_add_list = 添加列表
blocking_add_list_placeholder = https://example.com/list.txt
blocking_add_list_caption = 支持 Adblock Plus 或 hosts 格式的列表。运行脚本或改写页面的规则会计入数量，但不会生效。
blocking_add_checking = 正在检查列表…
blocking_add_failed = 无法添加此列表：{ $error }
blocking_add_done = { $rules ->
   *[other] 已添加“{ $title }”：{ $rules } 条规则。
}
blocking_add_unsupported = { $n ->
   *[other] { $n } 条规则无法应用。
}
blocking_remove_list = 移除
blocking_options = 选项
blocking_document_blocking = 拦截被拦截网站的页面
blocking_document_blocking_caption = 指向列表中网站的链接不会打开。
blocking_same_site = 拦截网站自身的请求
blocking_same_site_caption = 拦截网站自己投放的广告。拦截得更多，但会有更多网站无法正常运行。
blocking_restore_preset = 恢复预设
blocking_status_preparing = 正在准备过滤列表…
blocking_status_kept = 无法下载 { $lists }。将继续使用之前的列表。
blocking_reload_offer = 列表已更改。重新加载页面即可使用。
blocking_reload_page = 重新加载页面
blocking_this_site = 在此网站上拦截
blocking_site_default = 使用默认设置（{ $level }）
blocked_count = { $n ->
   *[other] 已拦截 { $n } 项
}
clear_history = 清除历史记录
clear_history_confirm = 清除所有浏览历史记录？此操作无法撤销。
clear_website_data = 清除网站数据
clear_website_data_caption = 移除 Cookie、缓存和已存储的网站数据，你将退出各网站的登录。
clear_website_data_confirm = 移除所有 Cookie 和网站数据？你将退出各网站的登录。
clear_bookmarks = 删除所有书签
clear_bookmarks_confirm = 删除所有书签？此操作无法撤销。

## Downloads, links to other apps, sign-in and error pages
downloads = 下载
download_confirm = 下载“{ $name }”？
download = 下载
search_downloads = 搜索下载
downloads_empty = 你下载的文件会显示在这里。
download_progress = { $received }/{ $total }
download_failed = 失败
download_canceled = 已取消
clear_downloads = 清除
remove_download = 从列表中移除
open_app_title = 在其他 App 中打开？
open_app_message = 此链接将在 Net Skip 之外打开：{ $url }
open = 打开
sign_in_title = 登录
sign_in_message = “{ $host }”要求输入名称和密码。
sign_in_failed = 该名称或密码未被接受。
user_name = 用户名
password = 密码
sign_in = 登录
error_offline_title = 你已离线
error_offline_message = 请连接互联网，然后重试。
error_host_title = 找不到网站
error_host_message = Net Skip 找不到服务器“{ $host }”。
error_timeout_title = 网站响应时间过长
error_timeout_message = “{ $host }”未及时响应。
error_connection_title = 无法连接
error_connection_message = Net Skip 无法连接到“{ $host }”。
error_secure_title = 无法安全连接
error_secure_message = Net Skip 无法与“{ $host }”建立安全连接。
error_certificate_title = 此连接非私密连接
error_certificate_message = “{ $host }”的安全证书不受信任。可能有人在冒充该网站以窃取你的信息。
error_crashed_title = 此页面已停止工作
error_crashed_message = 页面出现问题，已被关闭。
error_other_title = 无法打开此页面
error_other_message = Net Skip 无法打开“{ $host }”上的页面。
try_again = 重试
visit_anyway = 仍然访问网站
visit_anyway_title = 访问此网站？
visit_anyway_message = 你与“{ $host }”的连接将不是私密的。

## The user agent setting
settings_websites = 网站
user_agent = 用户代理
user_agent_caption = Net Skip 向网站表明的身份。更改后页面会重新加载。
user_agent_default = 默认
user_agent_engine = 内嵌网页视图
user_agent_custom = 自定用户代理

## Permission prompts
# What the system shows when a web page asks for a device feature (Day.toml [permissions]).
permission_camera = Net Skip 需要访问你的相机，以便你访问的网站在你授权后可用于视频通话、扫描二维码和拍摄照片。
permission_microphone = Net Skip 需要访问你的麦克风，以便你访问的网站在你授权后可用于语音和视频通话、语音搜索和录制音频。
permission_location_when_in_use = Net Skip 使用你的位置为你访问的网站提供基于位置的功能，例如在你授权后提供地图、本地搜索结果和天气信息。
permission_motion = Net Skip 使用动作和方向数据，以便你访问的网站在你授权后可提供能响应设备移动和倾斜的互动体验。
permission_NSSpeechRecognitionUsageDescription = Net Skip 使用语音识别，以便你无需键入即可通过语音输入网址、搜索词和表单内容。
