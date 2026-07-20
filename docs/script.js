(function () {
  'use strict';

  var currentPage = getCurrentPage();

  fetch('/sidebar.html')
    .then(function (r) { return r.text(); })
    .then(function (html) {
      document.getElementById('sidebar-container').innerHTML = html;
      highlightCurrentPage();
    })
    .catch(function () {
      // fallback: embed sidebar directly if fetch fails (e.g. file://)
      var sidebar = document.getElementById('sidebar-container');
      if (sidebar && !sidebar.hasChildNodes()) {
        sidebar.innerHTML = '<p style="padding:20px;color:#666;">Navigation unavailable</p>';
      } else {
        highlightCurrentPage();
      }
    });

  function getCurrentPage() {
    var path = window.location.pathname.replace(/\/+$/, '');
    var parts = path.split('/');
    var last = parts[parts.length - 1];

    if (!last || last === 'index' || last === 'index.html') {
      return 'index';
    }
    return last.replace(/\.html$/, '');
  }

  function highlightCurrentPage() {
    var links = document.querySelectorAll('.nav-link');
    links.forEach(function (link) {
      var page = link.getAttribute('data-page');
      if (page === currentPage) {
        link.classList.add('active');
      } else {
        link.classList.remove('active');
      }
    });
  }
})();
