// Copyright © The Daybrite Project · SPDX-License-Identifier: GPL-2.0-or-later
// A pretend tracking script. It only records that it ran.
(function () {
  (window.__loaded = window.__loaded || []).push('tracker-collect');
  window.__trackerQueue = [{ event: 'pageview', path: location.pathname }];
})();
