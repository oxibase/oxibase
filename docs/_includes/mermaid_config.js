{
  theme: (function() {
    const savedTheme = window.localStorage.getItem('theme');
    if (savedTheme === 'dark') return 'dark';
    if (savedTheme === 'blue') return 'neutral';
    if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) return 'dark';
    return 'neutral';
  })()
}