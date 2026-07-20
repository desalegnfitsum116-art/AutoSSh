(function () {
  'use strict';

  var currentPage = getCurrentPage();

  // Determine path from current page to docs root.
  // Pages in subdirectories need ../ prepended to reach root-level assets.
  var basePath = /\/pages\/.+\.html$/i.test(window.location.pathname) ? '../' : './';

  fetch(basePath + 'sidebar.html')
    .then(function (r) { return r.text(); })
    .then(function (html) {
      // Adjust sidebar link hrefs to be relative to current page
      var div = document.createElement('div');
      div.innerHTML = html;
      var links = div.querySelectorAll('.nav-link, .sidebar-logo');
      links.forEach(function (link) {
        var href = link.getAttribute('href');
        if (href && !href.startsWith('http') && !href.startsWith('/') && !href.startsWith('#')) {
          link.setAttribute('href', basePath + href);
        }
      });
      document.getElementById('sidebar-container').innerHTML = div.innerHTML;
      highlightCurrentPage();
    })
    .catch(function () {
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
