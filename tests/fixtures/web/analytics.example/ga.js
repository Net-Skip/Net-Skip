// Copyright © The Daybrite Project · SPDX-License-Identifier: GPL-2.0-or-later
// A pretend analytics script. It only records that it ran.
(function () {
  (window.__loaded = window.__loaded || []).push('analytics-ga');
  window.__analytics = { started: true };
})();
