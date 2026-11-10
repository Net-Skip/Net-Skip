// Copyright © The Daybrite Project · SPDX-License-Identifier: MPL-2.0
// The Daily Lantern's own script. It must always run, ad blocking or not.
(function () {
  (window.__loaded = window.__loaded || []).push('app');
  function ready() {
    document.documentElement.dataset.app = 'ready';
    var status = document.getElementById('app-status');
    if (status) status.textContent = 'Live';
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', ready);
  else ready();
  // For tests: the ids of the ad placeholders that are still visible on the page.
  window.__visibleAds = function () {
    return Array.prototype.filter.call(document.querySelectorAll('[data-test-ad]'), function (el) {
      var style = getComputedStyle(el);
      return style.display !== 'none' && style.visibility !== 'hidden' && el.getBoundingClientRect().height > 0;
    }).map(function (el) { return el.id; });
  };
})();
