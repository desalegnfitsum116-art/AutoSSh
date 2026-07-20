(function () {
  'use strict';

  const sidebar = document.getElementById('sidebar');
  const navLinks = document.querySelectorAll('.nav-link');
  const sections = document.querySelectorAll('.section');

  let activeSection = 'intro';

  function activateSection(sectionId) {
    activeSection = sectionId;

    sections.forEach(function (el) {
      el.classList.remove('active');
    });

    navLinks.forEach(function (link) {
      link.classList.remove('active');
    });

    var sectionEl = document.getElementById('section-' + sectionId);
    if (sectionEl) {
      sectionEl.classList.add('active');
    }

    var activeLink = document.querySelector(
      '.nav-link[data-section="' + sectionId + '"]'
    );
    if (activeLink) {
      activeLink.classList.add('active');
    }

    history.replaceState(null, '', '#' + sectionId);
    document.title = 'AutoSSH — ' + getSectionTitle(sectionId);
  }

  function getSectionTitle(sectionId) {
    var titles = {
      intro: 'Introduction',
      install: 'Installation',
      quickstart: 'Quick Start',
      config: 'Configuration',
      usage: 'Usage Guide',
      architecture: 'Architecture',
      security: 'Security',
      troubleshooting: 'Troubleshooting',
      faq: 'FAQ',
      contributing: 'Contributing',
      changelog: 'Changelog',
    };
    return titles[sectionId] || 'Documentation';
  }

  navLinks.forEach(function (link) {
    link.addEventListener('click', function (e) {
      e.preventDefault();
      var sectionId = link.getAttribute('data-section');
      if (sectionId) {
        activateSection(sectionId);
      }
    });
  });

  function handleHash() {
    var hash = window.location.hash.replace('#', '');
    if (hash && document.getElementById('section-' + hash)) {
      activateSection(hash);
    } else {
      activateSection('intro');
    }
  }

  window.addEventListener('hashchange', handleHash);

  handleHash();
})();
