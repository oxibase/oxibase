document.addEventListener("DOMContentLoaded", function() {
    // Target links that aren't oxibase OR localhost OR 0.0.0.0
    const selector = 'a[href^="http"]:not([href*="oxibase.xyz"]):not([href*="localhost"]):not([href*="0.0.0.0"])';
    const externalLinks = document.querySelectorAll(selector);

    externalLinks.forEach(link => {
        link.setAttribute('target', '_blank');
        link.setAttribute('rel', 'noopener noreferrer');
    });
});