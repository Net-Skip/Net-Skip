// Copyright © The Daybrite Project · SPDX-License-Identifier: MPL-2.0
// A pretend tracking script. It only records that it ran.
(function () {
  (window.__loaded = window.__loaded || []).push('tracker-collect');
  window.__trackerQueue = [{ event: 'pageview', path: location.pathname }];
})();
