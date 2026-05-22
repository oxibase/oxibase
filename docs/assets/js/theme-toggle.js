(function() {
  function getInitialTheme() {
    const savedTheme = localStorage.getItem('theme');
    if (savedTheme) {
      return savedTheme;
    }
    if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
      return 'dark';
    }
    return 'blue';
  }

  function applyTheme(theme) {
    // Only call setTheme if the current theme is different
    if (jtd.getTheme() !== theme) {
      jtd.setTheme(theme);
    }
    const toggleButton = document.getElementById('theme-toggle');
    if (toggleButton) {
      toggleButton.innerText = theme === 'dark' ? '🌙' : '☀️';
    }
  }

  jtd.onReady(function() {
    const currentTheme = getInitialTheme();
    
    // Apply the initial theme
    applyTheme(currentTheme);

    const toggleButton = document.getElementById('theme-toggle');
    if (toggleButton) {
      toggleButton.addEventListener('click', function() {
        const newTheme = jtd.getTheme() === 'dark' ? 'blue' : 'dark';
        applyTheme(newTheme);
        localStorage.setItem('theme', newTheme);
      });
    }
  });
})();
