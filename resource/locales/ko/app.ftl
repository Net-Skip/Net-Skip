# Copyright © The Daybrite Project
# SPDX-License-Identifier: MPL-2.0
#
# All app-owned UI text. Settings and About labels come from Day's catalog.
app_title = Net Skip
about_tagline = 방해하지 않는 웹 브라우저.

## Menus
menu_file = 파일
menu_edit = 편집
menu_view = 보기
menu_history = 방문 기록
menu_bookmarks = 북마크
menu_tabs = 탭

## Tabs
new_tab = 새 탭
new_private_tab = 새 비공개 탭
private_tab = 비공개 탭
close_tab = 탭 닫기
close_other_tabs = 다른 탭 닫기
duplicate_tab = 탭 복제
reopen_closed_tab = 닫은 탭 다시 열기
recently_closed = 최근에 닫은 탭
no_closed_tabs = 최근에 닫은 탭 없음
show_all_tabs = 모든 탭 보기
next_tab = 다음 탭
previous_tab = 이전 탭
tab_number = 탭 { $n }
last_tab = 마지막 탭
tab_count = { $n ->
   *[other] 탭 { $n }개
}
open_in_new_tab = 새 탭에서 열기
open_in_background_tab = 백그라운드 탭에서 열기
open_location = 위치 열기…
private_explainer = 비공개 브라우징: 이 탭에서 방문한 페이지는 방문 기록에 남지 않으며, 탭을 닫으면 탭도 기억하지 않습니다.

## Navigation
back = 뒤로
forward = 앞으로
reload = 페이지 새로고침
stop_loading = 로드 중지
home = 홈
share = 공유
more = 더 보기
done = 완료
edit = 편집
cancel = 취소
settings = 설정

## The address bar
address_placeholder = { $engine }에서 검색 또는 주소 입력
secure_connection = 보안 연결
clear_text = 지우기
search_with = { $engine }에서 검색
go_to_site = 사이트로 이동
switch_to_tab = 탭으로 전환 · { $site }
fill_in_search = 검색창에 넣기

## Find in page and zoom
find_on_page = 페이지에서 찾기…
find_placeholder = 페이지에서 찾기
find_previous = 이전 일치
find_next = 다음 일치
find_none = 찾을 수 없음
find_count = { $current }/{ $total }
text_size = 텍스트 크기
desktop_site = 데스크톱 사이트
request_desktop_site = 데스크톱 웹 사이트 요청
request_mobile_site = 모바일 웹 사이트 요청
hide_toolbar = 도구 막대 가리기
actual_size_percent = 실제 크기({ $percent }%)
page_menu = 페이지 메뉴
show_toolbar = 도구 막대 보기
zoom_in = 확대
zoom_out = 축소
actual_size = 실제 크기

## Bookmarks and history
show_library = 북마크 및 방문 기록
show_bookmarks = 북마크 보기
show_all_history = 모든 방문 기록 보기
bookmarks = 북마크
history = 방문 기록
add_bookmark = 북마크 추가
remove_bookmark = 북마크 제거
copy_link = 링크 복사
delete_history_item = 방문 기록에서 제거
search_bookmarks = 북마크 검색
search_history = 방문 기록 검색
bookmarks_empty = 페이지를 북마크하면 여기와 시작 페이지에 표시됩니다.
history_empty = 방문한 페이지가 여기에 표시됩니다.
no_matches = 검색과 일치하는 항목이 없습니다.
favorites = 즐겨찾기
favorites_empty = 북마크가 여기에 표시됩니다.
frequently_visited = 자주 방문한 사이트
today = 오늘
yesterday = 어제
monday = 월요일
tuesday = 화요일
wednesday = 수요일
thursday = 목요일
friday = 금요일
saturday = 토요일
sunday = 일요일
clear_history_button = 지우기…
clear_history_prompt = 지울 방문 기록 기간:
clear_last_hour = 지난 1시간
clear_today = 오늘
clear_two_days = 오늘 및 어제
clear_all_history = 모든 방문 기록
clear_history_message = 지우면 방문한 페이지가 방문 기록에서 제거됩니다.
delete = 삭제

## Settings
settings_search = 검색
settings_browsing = 브라우징
settings_address_bar = 주소 막대
settings_privacy = 개인정보 보호
search_engine = 검색 엔진
search_custom = 사용자 지정
search_custom_template = 검색 주소
search_custom_placeholder = https://example.com/search?q=%s
search_custom_caption = 결과 페이지의 주소로, 검색어가 들어갈 자리에 %s를 넣습니다.
search_suggestions = 검색 제안
search_suggestions_caption = 주소 막대에 입력하는 내용을 입력하는 동안 검색 엔진으로 보냅니다.
library_suggestions = Net Skip 제안
library_suggestions_caption = 입력하는 동안 방문 기록과 북마크에서 페이지를 제안합니다.
settings_tabs = 탭
address_at_top = 주소 막대를 상단에 표시
address_at_top_caption = 휴대폰에서 주소 막대를 페이지 아래가 아닌 위에 표시합니다.
open_links_background = 백그라운드에서 링크 열기
close_tabs = 탭 닫기
close_tabs_manually = 수동으로
close_tabs_day = 1일 후
close_tabs_week = 1주 후
close_tabs_month = 1개월 후
home_page = 홈페이지
home_page_placeholder = 시작 페이지
restore_tabs = 지난번 탭 다시 열기
show_frequent = 자주 방문한 사이트 보기
full_address = 웹 사이트 전체 주소 보기
upgrade_https = HTTPS로 업그레이드
upgrade_https_caption = 사이트가 허용하면 입력한 http:// 주소를 안전하게 로드합니다.
page_zoom = 페이지 확대/축소
blocking = 광고 및 추적기
blocking_level = 차단
blocking_off = 끔
blocking_light = 약하게
blocking_strong = 강하게
blocking_aggressive = 매우 강하게
blocking_caption = ‘약하게’는 대부분의 추적기와 일부 광고를 차단합니다. ‘강하게’는 더 많은 추적기와 광고를 차단하며, 일부 사이트가 제대로 작동하지 않을 수 있습니다. ‘매우 강하게’는 거의 모든 추적기와 광고를 차단하며, 사이트가 제대로 작동하지 않게 됩니다.
blocking_update = 지금 목록 업데이트
blocking_unavailable = 콘텐츠 차단을 사용할 수 없습니다.
blocking_status_off = 차단이 꺼져 있습니다.
blocking_status_waiting = 필터 목록 다운로드 중…
blocking_status_ready = { $lists ->
   *[other] 필터 목록 { $lists }개를 사용 중이며, 목록은 최신 상태로 유지됩니다.
}
blocking_status_failed = 필터 목록을 다운로드하지 못했습니다. Net Skip이 다시 시도합니다.
blocking_custom = 사용자 지정
blocking_filter_lists = 필터 목록
blocking_lists_caption = 목록을 켜거나 끄거나 옵션을 변경하면 차단이 ‘사용자 지정’으로 바뀝니다. 처음부터 다시 하려면 프리셋을 복원하세요.
blocking_lists_general = 목록
blocking_lists_regional = 지역 목록
blocking_lists_added = 추가한 목록
blocking_purpose_privacy = 추적기
blocking_purpose_ads = 광고
blocking_purpose_cookies = 쿠키 알림
blocking_purpose_annoyances = 성가신 요소
blocking_purpose_social = 소셜 위젯
blocking_purpose_regional = 지역 광고
blocking_purpose_security = 악성 코드 및 사기
blocking_list_not_downloaded = 다운로드되지 않음
blocking_list_failed = 다운로드 실패
blocking_list_stale = 업데이트 실패, 마지막 사본 사용 중
blocking_list_updated_now = 방금 업데이트됨
blocking_list_updated_minutes = { $n ->
   *[other] { $n }분 전에 업데이트됨
}
blocking_list_updated_hours = { $n ->
   *[other] { $n }시간 전에 업데이트됨
}
blocking_list_updated_days = { $n ->
   *[other] { $n }일 전에 업데이트됨
}
blocking_add_list = 목록 추가
blocking_add_list_placeholder = https://example.com/list.txt
blocking_add_list_caption = Adblock Plus 또는 hosts 형식의 목록을 사용할 수 있습니다. 스크립트를 실행하거나 페이지를 다시 쓰는 규칙은 개수에 포함되지만 적용되지는 않습니다.
blocking_add_checking = 목록 확인 중…
blocking_add_failed = 이 목록을 추가하지 못했습니다: { $error }
blocking_add_done = { $rules ->
   *[other] ‘{ $title }’ 추가됨: 규칙 { $rules }개
}
blocking_add_unsupported = { $n ->
   *[other] 규칙 { $n }개를 적용할 수 없습니다.
}
blocking_remove_list = 제거
blocking_options = 옵션
blocking_document_blocking = 차단된 사이트의 페이지 차단
blocking_document_blocking_caption = 목록에 있는 사이트로 연결되는 링크는 열리지 않습니다.
blocking_same_site = 사이트 자체 요청 차단
blocking_same_site_caption = 사이트가 직접 제공하는 광고를 차단합니다. 더 많이 차단하지만, 제대로 작동하지 않는 사이트도 늘어납니다.
blocking_restore_preset = 프리셋 복원
blocking_status_preparing = 필터 목록 준비 중…
blocking_status_kept = 다운로드하지 못한 목록: { $lists }. 이전 목록을 계속 사용합니다.
blocking_reload_offer = 목록이 변경되었습니다. 적용하려면 페이지를 새로고침하세요.
blocking_reload_page = 페이지 새로고침
blocking_this_site = 이 사이트에서 차단
blocking_site_default = 기본값 사용({ $level })
blocked_count = { $n ->
   *[other] 항목 { $n }개 차단됨
}
clear_history = 방문 기록 지우기
clear_history_confirm = 모든 방문 기록을 지우시겠습니까? 이 작업은 취소할 수 없습니다.
clear_website_data = 웹 사이트 데이터 지우기
clear_website_data_caption = 쿠키, 캐시, 저장된 사이트 데이터를 제거하며, 웹 사이트에서 로그아웃됩니다.
clear_website_data_confirm = 모든 쿠키와 웹 사이트 데이터를 제거하시겠습니까? 웹 사이트에서 로그아웃됩니다.
clear_bookmarks = 모든 북마크 삭제
clear_bookmarks_confirm = 모든 북마크를 삭제하시겠습니까? 이 작업은 취소할 수 없습니다.

## Permission prompts
# What the system shows when a web page asks for a device feature (Day.toml [permissions]).
permission_camera = Net Skip은 사용자가 권한을 허용하면 방문한 웹사이트가 영상 통화, QR 코드 스캔, 사진 촬영에 사용할 수 있도록 카메라 접근 권한이 필요합니다.
permission_microphone = Net Skip은 사용자가 권한을 허용하면 방문한 웹사이트가 음성·영상 통화, 음성 검색, 오디오 녹음에 사용할 수 있도록 마이크 접근 권한이 필요합니다.
permission_location_when_in_use = Net Skip은 사용자가 권한을 허용하면 방문한 웹사이트에 지도, 지역 검색 결과, 날씨 정보 등 위치 기반 기능을 제공하기 위해 위치를 사용합니다.
permission_motion = Net Skip은 사용자가 권한을 허용하면 방문한 웹사이트가 기기의 움직임과 기울임에 반응하는 인터랙티브 경험을 제공할 수 있도록 모션 및 방향 데이터를 사용합니다.
permission_NSSpeechRecognitionUsageDescription = Net Skip은 키보드로 입력하는 대신 음성으로 URL, 검색어, 양식 내용을 받아쓸 수 있도록 음성 인식을 사용합니다.
