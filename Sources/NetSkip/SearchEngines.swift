// SPDX-License-Identifier: GPL-2.0-or-later
import SwiftUI
import SkipWeb
import NetSkipModel

extension SearchEngine {
    static var defaultSearchEngines: [SearchEngine] { return [
        SearchEngine.duckduckgo,
        SearchEngine.google,
        SearchEngine.swisscows,
        SearchEngine.bing,
        SearchEngine.yahoo,
        SearchEngine.ecosia,
        SearchEngine.qwant,
        SearchEngine.startpage,
        SearchEngine.searx,
        SearchEngine.yandex,
        SearchEngine.baidu,
        SearchEngine.gigablast,
        SearchEngine.dogpile,
        SearchEngine.kagi,
    ] }

    /// The currently-selected search engine, or the first search engine in the list if it is unselected
    static func lookup(id: SearchEngine.ID) -> SearchEngine? {
        SearchEngine.defaultSearchEngines.first { engine in
            engine.id == id
        } ?? SearchEngine.defaultSearchEngines.first
    }
}


fileprivate func fetchSuggestions(from url: String) async throws -> Data {
    guard let suggestionURL = URL(string: url) else {
        throw URLError(.badURL)
    }
    let request = URLRequest(url: suggestionURL)
    let (data, response) = try await URLSession.shared.data(for: request)
    let code = (response as? HTTPURLResponse)?.statusCode

    if code != 200 {
        throw URLError(.badServerResponse)
    }

    return data
}

extension SearchEngine {
    public static let duckduckgo = SearchEngine(id: "duckduckgo", homeURL: "https://duckduckgo.com/", name: {
        NSLocalizedString("DuckDuckGo", bundle: .module, comment: "search engine name for DuckDuckGo")
    }) { q, l in
        "https://duckduckgo.com/?q=\(q)"
    } suggestions: { q in
        // curl 'https://duckduckgo.com/ac/?q=sail&kl=en'
        // [{"phrase":"sailor moon"},{"phrase":"sailing anarchy"},{"phrase":"sailrite"},{"phrase":"sailor"},{"phrase":"sailboats for sale"},{"phrase":"sailpoint"},{"phrase":"sailboat insurance"},{"phrase":"sailfish"}]
        // SKIP NOWARN
        let responses: [DDGResponse] = try JSONDecoder().decode([DDGResponse].self, from: await fetchSuggestions(from: "https://duckduckgo.com/ac/?q=\(q)"))
        return responses.map({ $0.phrase })
    }
}

private struct DDGResponse : Decodable {
    let phrase: String
}

extension SearchEngine {
    public static let swisscows = SearchEngine(id: "swisscows", homeURL: "https://swisscows.com/", name: {
        NSLocalizedString("Swisscows", bundle: .module, comment: "search engine name for Swisscows")
    }) { q, l in
        "https://swisscows.com/web?query=\(q)"
    } suggestions: { q in
        // "https://api.swisscows.com/suggest?query=\(q)&locale=\(l)"
        // &itemsCount=5
        // ["sailor moon","sailor","sailer verlag","sailfish","sailer"]
        // SKIP NOWARN
        return try JSONDecoder().decode([String].self, from: await fetchSuggestions(from: "https://api.swisscows.com/suggest?query=\(q)"))
    }
}

extension SearchEngine {
    public static let google = SearchEngine(id: "google", homeURL: "https://www.google.com/", name: {
        NSLocalizedString("Google", bundle: .module, comment: "search engine name for Google")
    }) { q, l in
        "https://www.google.com/search?q=\(q)"
    } suggestions: { q in
        // curl 'https://www.google.com/complete/search?q=sail&client=safari&hl=en'
        // ["sail",[["sailor moon","",[512,433]],["sail loft boston","",[512]],["sail loft","",[512]],["sailpoint","",[512,433]],["sails library network","",[512]],["sailor","",[512,433]],["sail biomedicines","",[512]],["sailor moon characters","",[512,433]],["sailfish","",[512,433]],["sailboat","",[512,433]]],{"k":1,"q":"XykCqol2X3ZvbGZ3h7vbebdvUug"}]
        // "https://www.google.com/complete/search?q=\(q)&client=safari&hl=\(l)"

        // TODO: Google search suggestions API returns a heterogeneous array of arrays, which is not easily representable from Decodable types; we may want to use JSONSerialization to parse the response and manually dig through the arrays for the suggestions
        nil
    }
}

extension SearchEngine {
    public static let bing = SearchEngine(id: "bing", homeURL: "https://www.bing.com/", name: {
        NSLocalizedString("Bing", bundle: .module, comment: "search engine name for Bing")
    }) { q, l in
        "https://www.bing.com/search?q=\(q)"
    } suggestions: { q in
        // seems to require an API key
        nil // "https://www.bing.com/AS/Suggestions?q=\(q)"
    }
}

extension SearchEngine {
    public static let yahoo = SearchEngine(id: "yahoo", homeURL: "https://search.yahoo.com/", name: {
        NSLocalizedString("Yahoo!", bundle: .module, comment: "search engine name for Yahoo")
    }) { q, l in
        "https://search.yahoo.com/search?p=\(q)"
    } suggestions: { q in
        nil // https://search.yahooapis.com/WebSearchService/V1/relatedSuggestion?appid=YahooDemo&output=json&query=sail
    }
}

extension SearchEngine {
    public static let yandex = SearchEngine(id: "yandex", homeURL: "https://yandex.com/", name: {
        NSLocalizedString("Yandex", bundle: .module, comment: "search engine name for Yandex")
    }) { q, l in
        "https://yandex.com/search/?text=\(q)"
    } suggestions: { q in
        nil // "https://suggest.yandex.net/suggest-ff.cgi?part=\(q)&uil=en&lid=1000000&clid=1000000&reqenc=utf-8&region=us"
    }
}

extension SearchEngine {
    public static let baidu = SearchEngine(id: "baidu", homeURL: "http://www.baidu.com/", name: {
        NSLocalizedString("Baidu", bundle: .module, comment: "search engine name for baidu")
    }) { q, l in
        "http://www.baidu.com/s?wd=\(q)"
    } suggestions: { q in
        nil
    }
}

extension SearchEngine {
    public static let ecosia = SearchEngine(id: "ecosia", homeURL: "https://www.ecosia.org/", name: {
        NSLocalizedString("Ecosia", bundle: .module, comment: "search engine name for Ecosia")
    }) { q, l in
        "https://www.ecosia.org/search?q=\(q)"
    } suggestions: { q in
        nil
    }
}

extension SearchEngine {
    public static let qwant = SearchEngine(id: "qwant", homeURL: "https://www.qwant.com/", name: {
        NSLocalizedString("Qwant", bundle: .module, comment: "search engine name for Qwant")
    }) { q, l in
        "https://www.qwant.com/?q=\(q)"
    } suggestions: { q in
        // {"status":"success","data":{"items":[{"value":"sailor moon","suggestType":0},{"value":"sailor","suggestType":0},{"value":"sailboat","suggestType":0},{"value":"sailpoint","suggestType":0},{"value":"sailrite","suggestType":0},{"value":"sailfish","suggestType":0},{"value":"sailboat insurance","suggestType":0}],"special":[]}}
        nil // "https://api.qwant.com/v3/suggest?q=\(q)&locale=\(l)"
    }
}

extension SearchEngine {
    public static let startpage = SearchEngine(id: "startpage", homeURL: "https://www.startpage.com/", name: {
        NSLocalizedString("StartPage", bundle: .module, comment: "search engine name for StartPage")
    }) { q, l in
        "https://www.startpage.com/do/dsearch?query=\(q)"
    } suggestions: { q in
        nil
    }
}

extension SearchEngine {
    public static let searx = SearchEngine(id: "searx", homeURL: "https://searx.me/", name: {
        NSLocalizedString("Searx", bundle: .module, comment: "search engine name for Searx")
    }) { q, l in
        "https://searx.me/?q=\(q)"
    } suggestions: { q in
        nil
    }
}

extension SearchEngine {
    public static let gigablast = SearchEngine(id: "gigablast", homeURL: "https://www.gigablast.com/", name: {
        NSLocalizedString("GigaBlast", bundle: .module, comment: "search engine name for GigaBlast")
    }) { q, l in
        "https://www.gigablast.com/search?q=\(q)"
    } suggestions: { q in
        nil
    }
}

extension SearchEngine {
    public static let dogpile = SearchEngine(id: "dogpile", homeURL: "https://www.dogpile.com/", name: {
        NSLocalizedString("Dogpile", bundle: .module, comment: "search engine name for Dogpile")
    }) { q, l in
        "https://www.dogpile.com/serp?q=\(q)"
    } suggestions: { q in
        nil // "https://www.dogpile.com/serp/suggestions.js?qc=QUERY&_=1646097468734"
    }
}

extension SearchEngine {
    public static let kagi = SearchEngine(id: "kagi", homeURL: "https://kagi.com/", name: {
        NSLocalizedString("Kagi", bundle: .module, comment: "search engine name for Kagi")
    }) { q, l in
        "https://kagi.com/search?q=\(q)"
    } suggestions: { q in
        // "https://kagi.com/api/autosuggest?q=\(q)"
        nil
    }
}

//extension SearchEngine {
//    public static let XXX = SearchEngine(id: "XXX", homeURL: "XXX", name: {
//        NSLocalizedString("XXX", bundle: .module, comment: "search engine name for XXX")
//    }) { q, l in
//        "XXX"
//    } suggestions: { q in
//        "XXX"
//    }
//}

