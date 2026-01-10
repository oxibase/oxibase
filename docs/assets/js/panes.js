document.addEventListener('DOMContentLoaded', function() {
  const panesContainer = document.querySelector('.panes-container');

  // Intercept all internal links
  document.addEventListener('click', function(e) {
    const link = e.target.closest('a');
    if (!link) return;

    // Skip links in the main navigation sidebar
    if (link.closest('.site-nav')) return;

    // Check if internal link
    const href = link.getAttribute('href');
    const isInternal = href && !href.startsWith('http') && !href.startsWith('mailto:') && !href.startsWith('#') && !link.hasAttribute('download');

    if (!isInternal) return;

    e.preventDefault();

    // Find current pane
    const currentPane = e.target.closest('.pane');
    if (!currentPane) return;

    const panes = Array.from(panesContainer.children);
    const currentIndex = panes.indexOf(currentPane);

    // Remove panes after current
    panes.slice(currentIndex + 1).forEach(pane => pane.remove());

    // Fetch and append new pane
    fetch(link.href)
      .then(response => response.text())
      .then(html => {
        const parser = new DOMParser();
        const doc = parser.parseFromString(html, 'text/html');
        const mainContent = doc.getElementById('main-content');
        if (!mainContent) return;

        // Create new pane
        const newPane = document.createElement('div');
        newPane.className = 'pane';
        newPane.innerHTML = `
          <div class="main-content-wrap">
            ${mainContent.outerHTML}
          </div>
        `;

        panesContainer.appendChild(newPane);

        // Scroll to new pane
        newPane.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'end' });

        // Update URL
        history.pushState(null, '', link.href);
      })
      .catch(console.error);
  });

  // Handle browser back/forward
  window.addEventListener('popstate', function() {
    const panes = panesContainer.children;
    if (panes.length > 1) {
      panes[panes.length - 1].remove();
    }
  });
});