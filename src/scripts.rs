// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
//! The scripts Net Skip runs in pages: reading the page's icons and finding text.
//! Each is one expression; day-piece-webview evaluates it and returns its value as JSON.

/// The page's icons (`<link rel=icon>`, `apple-touch-icon`) and theme color, as
/// `{"icons":[{"href","rel","sizes","type"}…],"theme":"…"}`.
pub const PAGE_INFO: &str = r#"(function(){
  var icons = [];
  document.querySelectorAll('link[rel]').forEach(function(l){
    var rel = (l.getAttribute('rel') || '').toLowerCase();
    if (rel.indexOf('icon') < 0 || !l.href) return;
    icons.push({href: l.href, rel: rel, sizes: l.getAttribute('sizes') || '', type: l.getAttribute('type') || ''});
  });
  var theme = document.querySelector('meta[name="theme-color"]');
  return {icons: icons, theme: theme ? theme.content : ''};
})()"#;

/// The find-in-page engine, installed on first use: matches are wrapped in `<mark>` elements
/// (case-insensitive, across the page's visible text), the current one is colored and scrolled
/// into view. Engines differ in their native find APIs, so the page does the work itself, as
/// Firefox for iOS and DuckDuckGo for iOS do.
const FIND_ENGINE: &str = r#"(window.__netskipFind = window.__netskipFind || (function(){
  var marks = [], current = -1, CLS = '__netskip_find';
  var style = document.createElement('style');
  style.textContent = 'mark.' + CLS + '{background:#ffe066;color:inherit;border-radius:2px}' +
    'mark.' + CLS + '.current{background:#ff9632}';
  function clear(){
    marks.forEach(function(m){
      var p = m.parentNode; if (!p) return;
      p.replaceChild(document.createTextNode(m.textContent), m); p.normalize();
    });
    marks = []; current = -1;
  }
  function visible(n){
    var e = n.parentElement;
    while (e) {
      var t = e.tagName;
      if (t === 'SCRIPT' || t === 'STYLE' || t === 'NOSCRIPT' || t === 'TEXTAREA') return false;
      e = e.parentElement;
    }
    return true;
  }
  function search(q){
    clear();
    if (!q) return state();
    if (!style.parentNode) (document.head || document.documentElement).appendChild(style);
    var needle = q.toLowerCase(), walker = document.createTreeWalker(document.body || document.documentElement, NodeFilter.SHOW_TEXT), nodes = [];
    while (walker.nextNode()) { if (visible(walker.currentNode)) nodes.push(walker.currentNode); }
    nodes.forEach(function(node){
      var text = node.nodeValue, lower = text.toLowerCase(), at = lower.indexOf(needle);
      if (at < 0) return;
      var frag = document.createDocumentFragment(), last = 0;
      while (at >= 0 && marks.length < 1000) {
        frag.appendChild(document.createTextNode(text.slice(last, at)));
        var m = document.createElement('mark'); m.className = CLS;
        m.textContent = text.slice(at, at + needle.length);
        frag.appendChild(m); marks.push(m);
        last = at + needle.length; at = lower.indexOf(needle, last);
      }
      frag.appendChild(document.createTextNode(text.slice(last)));
      node.parentNode.replaceChild(frag, node);
    });
    if (marks.length) select(0);
    return state();
  }
  function select(i){
    if (!marks.length) return state();
    if (current >= 0 && marks[current]) marks[current].classList.remove('current');
    current = (i + marks.length) % marks.length;
    marks[current].classList.add('current');
    marks[current].scrollIntoView({block: 'center', inline: 'nearest'});
    return state();
  }
  function state(){ return {current: marks.length ? current + 1 : 0, total: marks.length}; }
  return {
    search: search,
    next: function(){ return select(current + 1); },
    previous: function(){ return select(current - 1); },
    clear: function(){ clear(); return state(); }
  };
})())"#;

/// Find `query` in the page, highlighting every match and selecting the first.
pub fn find(query: &str) -> String {
    format!("{FIND_ENGINE}.search({})", js_string(query))
}

/// The next match.
pub fn find_next() -> String {
    format!("{FIND_ENGINE}.next()")
}

/// The previous match.
pub fn find_previous() -> String {
    format!("{FIND_ENGINE}.previous()")
}

/// Remove the highlights.
pub fn find_clear() -> String {
    format!("{FIND_ENGINE}.clear()")
}

/// Parse a find reply `{"current":n,"total":m}`.
pub fn parse_matches(json: &str) -> (u32, u32) {
    let v: serde_json::Value = serde_json::from_str(json).unwrap_or_default();
    let n = |k: &str| v.get(k).and_then(|x| x.as_u64()).unwrap_or(0) as u32;
    (n("current"), n("total"))
}

/// A JavaScript string literal for `s`.
fn js_string(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_quotes_the_query() {
        let script = find("it's \"quoted\" </script>");
        assert!(script.ends_with(r#".search("it's \"quoted\" </script>")"#));
    }

    #[test]
    fn matches_parse_leniently() {
        assert_eq!(parse_matches(r#"{"current":2,"total":7}"#), (2, 7));
        assert_eq!(parse_matches("null"), (0, 0));
        assert_eq!(parse_matches("garbage"), (0, 0));
    }
}
