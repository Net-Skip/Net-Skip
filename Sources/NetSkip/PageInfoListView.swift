// SPDX-License-Identifier: GPL-2.0-or-later
import SwiftUI
import SkipWeb
import NetSkipModel


struct PageInfoListView<ToolbarItems : ToolbarContent> : View {
    let type: PageInfo.PageType
    let store: WebBrowserStore
    let onSelect: (PageInfo) -> ()
    let onDelete: ([PageInfo]) -> ()
    let onOpenInNewTab: (PageInfo) -> ()
    /// Optional bulk action — when non-nil, an "Open All in Tabs" button is
    /// rendered at the top of the list. Used by the Favorites view to let
    /// the user fire off every saved bookmark as separate tabs in one
    /// gesture. History intentionally omits this since it would create
    /// hundreds of tabs.
    let onOpenAllInTabs: (([PageInfo]) -> ())?
    let toolbarItems: () -> ToolbarItems
    @State var items: [PageInfo] = []
    @State var searchText: String = ""
    @Environment(\.dismiss) var dismiss

    /// Case-insensitive substring filter over title + URL. Empty query
    /// returns all items so search becomes the no-op pass-through.
    var filteredItems: [PageInfo] {
        let trimmed = searchText.trimmingCharacters(in: .whitespacesAndNewlines)
        if trimmed.isEmpty {
            return items
        }
        let needle = trimmed.lowercased()
        return items.filter { item in
            if let title = item.title, title.lowercased().contains(needle) {
                return true
            }
            if let url = item.url, url.lowercased().contains(needle) {
                return true
            }
            return false
        }
    }

    func loadPageInfoItems() {
        let items = trying {
            try store.loadItems(type: type, ids: [])
        }
        if let items = items {
            self.items = items
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            if !items.isEmpty {
                searchField
                if onOpenAllInTabs != nil {
                    openAllButton
                }
            }
            if items.isEmpty {
                Text("No Items", bundle: .module, comment: "sheet placeholder text when there are no items available")
                    .font(.title)
                    .opacity(0.8)
                    .frame(maxHeight: .infinity)
            } else if filteredItems.isEmpty {
                Text("No matches", bundle: .module, comment: "shown in the History/Favorites list when the active search filter has zero results")
                    .font(.title3)
                    .opacity(0.7)
                    .frame(maxHeight: .infinity)
            } else {
                itemsListView
            }
        }
        .onAppear {
            loadPageInfoItems()
        }
        .onChange(of: type) {
            loadPageInfoItems()
        }
        .toolbar {
            #if !SKIP
            // Skip is unable to match this API call to determine whether it results in a View. Consider adding additional type information
            self.toolbarItems()
            #else
            ToolbarItem(placement: .bottomBar) {
                Button {
                    logger.log("clearing \(type.tableName)")
                    trying {
                        try store.removeItems(type: type, ids: [])
                        onDelete(items)
                        loadPageInfoItems()
                    }
                    dismiss()
                } label: {
                    type.clearTitle.bold()
                }
                .buttonStyle(.plain)
                .disabled(items.isEmpty)
            }
            #endif
        }
        .navigationTitle(type.navigationTitle)
        #if !os(macOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
    }

    @ViewBuilder
    var openAllButton: some View {
        Button {
            let action = onOpenAllInTabs
            let toOpen = filteredItems
            dismiss()
            action?(toOpen)
        } label: {
            HStack(spacing: 8) {
                Image("plus.square.on.square", bundle: .module)
                Text("Open All in Tabs", bundle: .module, comment: "button label that opens every filtered Favorites row in a separate tab")
                Spacer()
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 10)
            .frame(maxWidth: .infinity)
            .background(Color.accentColor.opacity(0.12))
            .foregroundStyle(Color.accentColor)
            .cornerRadius(10)
        }
        .buttonStyle(.plain)
        .padding(.horizontal, 12)
        .padding(.bottom, 6)
        .accessibilityIdentifier("button.pageInfo.openAllInTabs")
    }

    @ViewBuilder
    var searchField: some View {
        HStack(spacing: 8) {
            Image("magnifyingglass", bundle: .module)
                .foregroundStyle(.secondary)
                .font(.system(size: 14))
            TextField(text: $searchText, prompt: Text("Search", bundle: .module, comment: "placeholder text for the History / Favorites filter field")) {
                Text("Search", bundle: .module, comment: "accessibility label for the History / Favorites filter field")
            }
            .textFieldStyle(.plain)
            .font(.system(size: 15))
            #if !SKIP
            .autocorrectionDisabled(true)
            .textInputAutocapitalization(.never)
            #endif
            .accessibilityIdentifier("field.pageInfo.search")

            if !searchText.isEmpty {
                Button {
                    searchText = ""
                } label: {
                    Image("xmark.circle.fill", bundle: .module)
                        .foregroundStyle(.secondary)
                }
                .buttonStyle(.plain)
                .accessibilityIdentifier("button.pageInfo.search.clear")
                .accessibilityLabel(Text("Clear search", bundle: .module, comment: "accessibility label for the clear-search button in the History/Favorites list"))
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(Color.gray.opacity(0.12))
        .cornerRadius(10)
        .padding(.horizontal, 12)
        .padding(.top, 8)
        .padding(.bottom, 4)
    }

    var itemsListView: some View {
        List {
            ForEach(filteredItems) { item in
                Button(action: {
                    dismiss()
                    onSelect(item)
                }, label: {
                    VStack(alignment: .leading) {
                        itemTitle(item: item)
                            .font(.body)
                            .lineLimit(1)
                        #if !SKIP
                        // SKIP TODO: formatted
//                                Text(item.date.formatted())
//                                    .font(.body)
//                                    .lineLimit(1)
                        #endif
                        Text(item.url ?? "")
                            .font(.caption)
                            .foregroundStyle(Color.gray)
                            .lineLimit(1)
                            #if !SKIP
                            .truncationMode(.middle)
                            #endif
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                    #if !SKIP
                    .contentShape(Rectangle()) // needed to make the tap target fill the area
                    #endif
                })
                .buttonStyle(.plain)
                .contextMenu {
                    Button {
                        dismiss()
                        onOpenInNewTab(item)
                    } label: {
                        Label {
                            Text("Open in New Tab", bundle: .module, comment: "context menu label on a History/Favorites row that opens the URL in a fresh background tab")
                        } icon: {
                            Image("plus.square.on.square", bundle: .module)
                        }
                    }
                    .accessibilityIdentifier("menu.pageInfo.openInNewTab")
                }
            }
            .onDelete { offsets in
                // Index into the visible (filtered) list — otherwise a
                // swipe-to-delete on a filtered row would delete the
                // wrong entry from the underlying store.
                let visible = filteredItems
                let deleteItems = offsets.map({ visible[$0] })
                let ids = deleteItems.map(\.id)
                logger.log("deleting \(type.tableName) items: \(ids)")
                trying {
                    try store.removeItems(type: type, ids: Set(ids))
                }
                onDelete(deleteItems)
                loadPageInfoItems()
            }
        }
    }

    func itemTitle(item: PageInfo) -> some View {
        if let title = item.title {
            return Text(title)
        } else {
            return Text("Empty Page", bundle: .module, comment: "item list title for a page with no title")
        }
    }
}

extension PageInfo.PageType {
    var navigationTitle: Text {
        switch self {
        case .history: return Text("History", bundle: .module, comment: "history page list sheet title")
        case .favorite: return Text("Favorites", bundle: .module, comment: "favorites page list sheet title")
        case .active: return Text("Tabs", bundle: .module, comment: "active tabs sheet title")
        }
    }

    var clearTitle: Text {
        switch self {
        case .history: return Text("Clear History", bundle: .module, comment: "history page remove all sheet title")
        case .favorite: return Text("Remove all Favorites", bundle: .module, comment: "favorites page remove all sheet title")
        case .active: return Text("Close All", bundle: .module, comment: "active tabs remove all title")
        }
    }
}
