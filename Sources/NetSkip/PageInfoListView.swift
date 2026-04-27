// SPDX-License-Identifier: GPL-2.0-or-later
import SwiftUI
import SkipWeb
import NetSkipModel


struct PageInfoListView<ToolbarItems : ToolbarContent> : View {
    let type: PageInfo.PageType
    let store: WebBrowserStore
    let onSelect: (PageInfo) -> ()
    let onDelete: ([PageInfo]) -> ()
    let toolbarItems: () -> ToolbarItems
    @State var items: [PageInfo] = []
    @Environment(\.dismiss) var dismiss

    func loadPageInfoItems() {
        let items = trying {
            try store.loadItems(type: type, ids: [])
        }
        if let items = items {
            self.items = items
        }
    }

    var body: some View {
        Group {
            if items.isEmpty {
                Text("No Items", bundle: .module, comment: "sheet placeholder text when there are no items available")
                    .font(.title)
                    .opacity(0.8)
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

    var itemsListView: some View {
        List {
            ForEach(items) { item in
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
            }
            .onDelete { offsets in
                let deleteItems = offsets.map({
                    items[$0]
                })
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
