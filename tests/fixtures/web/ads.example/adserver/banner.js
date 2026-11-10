// Copyright © The Daybrite Project · SPDX-License-Identifier: MPL-2.0
// A pretend ad server script. It records that it ran and fills the leaderboard slot.
(function () {
  (window.__loaded = window.__loaded || []).push('ads-banner');
  function fill() {
    var slot = document.getElementById('div-gpt-ad-1');
    if (!slot) return;
    slot.setAttribute('data-filled-by', 'ads-banner');
    var creative = slot.querySelector('.creative');
    if (creative) creative.textContent = 'Brightline Ferries · Winter fares to the islands';
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', fill);
  else fill();
})();
