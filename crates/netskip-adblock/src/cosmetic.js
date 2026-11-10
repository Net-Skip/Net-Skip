// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0

// Element hiding for netskip-adblock. Inject at document start into every frame. The script
// asks the host for the page's hiding rules, then reports the classes and ids that appear in
// the DOM so the host can answer with the generic rules that match them. Each answer is JSON
// data, `{"hide": [...], "generichide": bool}`, returned as the reply to the message
// (`window.dayMessage(name, text)` resolves with it) and applied here. A host without replies
// may instead evaluate `window.__dayAdblock.apply(answer)` in the frame.
// Message format: docs/adblock.md, "The cosmetic script".
(() => {
  "use strict";
  if (window.__dayAdblock) return;
  const CHANNEL = "dayAdblock";
  const MAX_BATCH = 500;
  const seenClasses = new Set();
  const seenIds = new Set();
  const applied = new Set();
  let classes = [];
  let ids = [];
  let timer = 0;
  let generichide = false;
  let style = null;

  function apply(answer) {
    if (!answer) return;
    if (answer.generichide) {
      generichide = true;
      classes = [];
      ids = [];
    }
    if (Array.isArray(answer.hide)) hide(answer.hide);
  }

  function send(message) {
    const text = JSON.stringify(Object.assign({ v: 1, url: location.href }, message));
    try {
      // day-piece-webview's message bridge answers with the reply.
      if (typeof window.dayMessage === "function") {
        return window.dayMessage(CHANNEL, text).then(
          (reply) => {
            if (reply) apply(JSON.parse(reply));
          },
          () => {},
        );
      }
      const webkit = window.webkit && window.webkit.messageHandlers;
      if (webkit && webkit[CHANNEL]) return webkit[CHANNEL].postMessage(text);
      const listener = window[CHANNEL];
      if (listener && typeof listener.postMessage === "function") return listener.postMessage(text);
      const webview2 = window.chrome && window.chrome.webview;
      if (webview2) return webview2.postMessage(text);
      if (typeof window.__dayAdblockSend === "function") return window.__dayAdblockSend(text);
    } catch (e) {
      // A host without a channel leaves the page as it is.
    }
  }

  function sheet() {
    if (style && style.isConnected) return style;
    const root = document.head || document.documentElement;
    if (!root) return null;
    style = style || document.createElement("style");
    style.setAttribute("data-day-adblock", "");
    root.appendChild(style);
    return style;
  }

  // Selectors waiting to be applied, and the timer that applies them. Every change to the sheet
  // restyles the whole page, so answers are gathered and applied together at most every 250 ms:
  // on a heavy page, applying each answer as it came kept the page's main thread busy.
  let pendingCss = "";
  let applyTimer = 0;

  function applyPending() {
    applyTimer = 0;
    if (!pendingCss) return;
    const el = sheet();
    if (!el) {
      document.addEventListener("DOMContentLoaded", applyPending, { once: true });
      return;
    }
    el.appendChild(document.createTextNode(pendingCss));
    pendingCss = "";
  }

  // One rule per selector: a selector the browser does not understand drops only its own rule.
  function hide(selectors) {
    for (const s of selectors) {
      if (typeof s !== "string" || applied.has(s)) continue;
      applied.add(s);
      pendingCss += s + "{display:none!important}\n";
    }
    if (pendingCss && !applyTimer) applyTimer = setTimeout(applyPending, applied.size > 0 && !style ? 0 : 250);
  }

  function flush() {
    timer = 0;
    while (!generichide && (classes.length || ids.length)) {
      send({ type: "ids", classes: classes.splice(0, MAX_BATCH), ids: ids.splice(0, MAX_BATCH) });
    }
    classes = [];
    ids = [];
  }

  function note(el) {
    if (generichide || el.nodeType !== 1) return;
    const id = el.id;
    if (id && !seenIds.has(id)) {
      seenIds.add(id);
      ids.push(id);
    }
    const list = el.classList;
    if (list) {
      for (const c of list) {
        if (!seenClasses.has(c)) {
          seenClasses.add(c);
          classes.push(c);
        }
      }
    }
    if (!timer && (classes.length || ids.length)) timer = setTimeout(flush, 250);
  }

  function scan(root) {
    if (generichide || !root || root.nodeType !== 1) return;
    note(root);
    for (const el of root.querySelectorAll("[id],[class]")) note(el);
  }

  Object.defineProperty(window, "__dayAdblock", {
    value: Object.freeze({ apply }),
  });

  send({ type: "page" });
  new MutationObserver((records) => {
    for (const r of records) {
      if (r.type === "attributes") note(r.target);
      else for (const n of r.addedNodes) scan(n);
    }
  }).observe(document, { subtree: true, childList: true, attributes: true, attributeFilter: ["class", "id"] });
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => scan(document.documentElement), { once: true });
  } else {
    scan(document.documentElement);
  }
})();
