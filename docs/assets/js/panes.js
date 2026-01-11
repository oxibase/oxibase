document.addEventListener("DOMContentLoaded", function () {
  // Configuration constants
  const CONFIG = {
    mediaQuery: "(min-width: 50rem)",
    timeouts: {
      initialLayout: 500,
      layoutUpdate: 50,
      copyReset: 4000,
    },
    widths: {
      minCardWidth: 350,
      paddingLeft: 32,
      paddingRight: 32,
    },
    selectors: {
      panesContainer: ".panes-container",
      mainScroller: ".main",
      pane: ".pane",
      paneHeader: ".pane-header",
      paneBody: ".pane-body",
      paneContent: ".pane-content",
      siteNav: ".site-nav",
      mermaidElements: ".language-mermaid",
      codeBlocks:
        "div.highlighter-rouge, div.listingblock > div.content, figure.highlight",
      wideViewToggle: "#wide-view-toggle",
      mainContent: ".main-content-wrap",
      titleElements: "h1, h2",
    },
    classes: {
      hasTransition: "has-transition",
      wideView: "wide-view",
      copyIcon: "copy-icon",
    },
    sessionKeys: {
      wideView: "wide-view",
    },
    svgs: {
      copied:
        '<svg viewBox="0 0 24 24" class="copy-icon"><use xlink:href="#svg-copied"></use></svg>',
      copy: '<svg viewBox="0 0 24 24" class="copy-icon"><use xlink:href="#svg-copy"></use></svg>',
    },
  };

  // Utility functions
  const Utils = {
    /**
     * Safely query a selector, returning null if not found
     * @param {string} selector - CSS selector
     * @param {Element} [context=document] - Context element
     * @returns {Element|null}
     */
    safeQuerySelector(selector, context = document) {
      try {
        return context.querySelector(selector);
      } catch (error) {
        console.error(`Invalid selector: ${selector}`, error);
        return null;
      }
    },

    /**
     * Safely query all selectors, returning empty NodeList if not found
     * @param {string} selector - CSS selector
     * @param {Element} [context=document] - Context element
     * @returns {NodeList}
     */
    safeQuerySelectorAll(selector, context = document) {
      try {
        return context.querySelectorAll(selector);
      } catch (error) {
        console.error(`Invalid selector: ${selector}`, error);
        return [];
      }
    },

    /**
     * Debounce function calls
     * @param {Function} func - Function to debounce
     * @param {number} wait - Wait time in ms
     * @returns {Function}
     */
    debounce(func, wait) {
      let timeout;
      return function executedFunction(...args) {
        const later = () => {
          clearTimeout(timeout);
          func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
      };
    },

    /**
     * Add code copy buttons to code blocks
     */
    addCodeCopyButtons() {
      if (!window.isSecureContext) {
        console.log(
          "Window does not have a secure context, therefore code clipboard copy functionality will not be available. For more details see https://web.dev/async-clipboard/#security-and-permissions",
        );
        return;
      }

      const codeBlocks = this.safeQuerySelectorAll(CONFIG.selectors.codeBlocks);
      const svgCopied = CONFIG.svgs.copied;
      const svgCopy = CONFIG.svgs.copy;

      codeBlocks.forEach((codeBlock) => {
        const copyButton = document.createElement("button");
        let timeout = null;
        copyButton.type = "button";
        copyButton.ariaLabel = "Copy code to clipboard";
        copyButton.innerHTML = svgCopy;
        codeBlock.appendChild(copyButton);

        copyButton.addEventListener("click", () => {
          if (timeout === null) {
            try {
              const code = (
                codeBlock.querySelector("pre:not(.lineno, .highlight)") ||
                codeBlock.querySelector("code")
              ).innerText;
              window.navigator.clipboard.writeText(code);

              copyButton.innerHTML = svgCopied;

              timeout = setTimeout(() => {
                copyButton.innerHTML = svgCopy;
                timeout = null;
              }, CONFIG.timeouts.copyReset);
            } catch (error) {
              console.error("Failed to copy code:", error);
            }
          }
        });
      });
    },

    /**
     * Render Mermaid diagrams in a given element
     * @param {Element} element - Element containing Mermaid code
     */
    async renderMermaid(element) {
      if (!window.mermaid || typeof window.mermaid.render !== "function") {
        console.warn("Mermaid not available or render function not found");
        return;
      }

      try {
        const id = "mermaid-" + Math.random().toString(36).substr(2, 9);
        const { svg } = await window.mermaid.render(
          id,
          element.textContent.trim(),
        );
        // Replace the <pre><code> with the SVG
        element.parentElement.outerHTML = svg;
      } catch (error) {
        console.error("Mermaid render error:", error);
      }
    },

    /**
     * Update the active navigation item based on the current URL
     * @param {string} url - The current URL to match (defaults to window.location.href)
     */
    updateActiveNav(url = window.location.href) {
      // Remove .active from all nav items and links
      const allItems = document.querySelectorAll(".site-nav .nav-list-item");
      allItems.forEach((item) => item.classList.remove("active"));
      const allLinks = document.querySelectorAll(".site-nav .nav-list a");
      allLinks.forEach((link) => link.classList.remove("active"));

      const currentPathname = new URL(url).pathname;

      // If home, do nothing
      if (currentPathname === "/") {
        return;
      }

      let currentLink = null;

      // First, try to find and restore the <a> with missing href (for navigation events)
      currentLink = Array.from(allLinks).find(
        (link) => !link.hasAttribute("href"),
      );
      if (currentLink) {
        currentLink.href = url; // Restore href
      } else {
        // Fallback: find by exact pathname match (for fresh reload/direct access)
        currentLink = Array.from(allLinks).find((link) => {
          try {
            return new URL(link.href).pathname === currentPathname;
          } catch {
            return false; // Invalid URL
          }
        });
      }

      // If a link found, set .active on it and ancestors
      if (currentLink) {
        currentLink.classList.add("active");
        let item = currentLink.closest(".nav-list-item");
        while (item) {
          item.classList.add("active");
          item = item.parentElement
            ? item.parentElement.closest(".nav-list-item")
            : null;
        }
      }
    },
  };

  // Sub-classes for modular functionality
  class MediaQueryHandler {
    constructor(paneSystem) {
      this.paneSystem = paneSystem;
      this.mqMd = window.matchMedia(CONFIG.mediaQuery);
      this.isWideView = false;
      this.wideViewToggle = Utils.safeQuerySelector(
        CONFIG.selectors.wideViewToggle,
      );
    }

    init() {
      if (this.mqMd.matches) {
        this.setupWideView();
        this.setupResizeHandler();
        this.paneSystem.enablePaneSystem();
      } else {
        this.disableForSmallScreens();
        this.paneSystem.disablePaneSystem();
      }
    }

    setupWideView() {
      if (!this.wideViewToggle) return;

      this.wideViewToggle.style.display = ""; // Reset to CSS default
      // Load saved state (per session)
      this.isWideView =
        sessionStorage.getItem(CONFIG.sessionKeys.wideView) === "true";
      this.wideViewToggle.checked = this.isWideView;
      if (this.isWideView) {
        document.body.classList.add(CONFIG.classes.wideView);
      }

      // Handle toggle
      this.wideViewToggle.addEventListener("change", (e) =>
        this.handleToggle(e),
      );
    }

    handleToggle(event) {
      const enabled = event.target.checked;
      sessionStorage.setItem(CONFIG.sessionKeys.wideView, enabled);
      this.isWideView = enabled;
      if (enabled) {
        document.body.classList.add(CONFIG.classes.wideView);
      } else {
        // When disabling manual, apply automatic if overflow
        if (
          this.paneSystem.panesContainer.scrollWidth >
          this.paneSystem.panesContainer.clientWidth
        ) {
          document.body.classList.add(CONFIG.classes.wideView);
        } else {
          document.body.classList.remove(CONFIG.classes.wideView);
        }
      }
    }

    setupResizeHandler() {
      window.addEventListener(
        "resize",
        Utils.debounce(() => this.paneSystem.layoutManager.updateLayout(), 100),
      );
    }

    checkOverflowAndEnableWideView() {
      // Check if panes overflow and enable wide view (only if manual toggle is off)
      if (
        !this.isWideView &&
        this.paneSystem.panesContainer.scrollWidth >
          this.paneSystem.panesContainer.clientWidth
      ) {
        document.body.classList.add(CONFIG.classes.wideView);
        document.body.classList.add(CONFIG.classes.hasTransition);
      }
    }

    removeWideViewIfNoOverflow() {
      // Check if still overflow after removing (only if manual toggle is off)
      if (
        !this.isWideView &&
        this.paneSystem.panesContainer.scrollWidth <=
          this.paneSystem.panesContainer.clientWidth
      ) {
        document.body.classList.remove(CONFIG.classes.wideView);
        document.body.classList.remove(CONFIG.classes.hasTransition);
      }
    }

    disableForSmallScreens() {
      // Disable on small screens - remove classes and reset
      document.body.classList.remove(
        CONFIG.classes.hasTransition,
        CONFIG.classes.wideView,
      );
      // Reset last card width
      this.paneSystem.panesContainer.style.setProperty(
        "--last-card-width",
        "var(--card-width)",
      );
      // Hide toggle
      if (this.wideViewToggle) {
        this.wideViewToggle.style.display = "none";
      }
    }

    onMediaQueryChange() {
      if (this.mqMd.matches) {
        this.setupWideView();
        this.setupResizeHandler();
        this.paneSystem.enablePaneSystem();
      } else {
        this.disableForSmallScreens();
        this.paneSystem.disablePaneSystem();
      }
    }
  }

  class LinkInterceptor {
    constructor(paneSystem) {
      this.paneSystem = paneSystem;
      this.enabled = false;
      this.initialized = false;
    }

    init() {
      if (this.initialized) return;
      document.addEventListener("click", (e) => this.handleClick(e));
      this.initialized = true;
    }

    handleClick(event) {
      if (!this.enabled) return;
      const link = event.target.closest("a");
      if (!link) return;

      // If link is to an ID on the same page, handle it manually without adding to history
      if (
        link.hash &&
        link.origin === window.location.origin &&
        link.pathname === window.location.pathname &&
        link.search === window.location.search
      ) {
        event.preventDefault();
        const target = document.querySelector(link.hash);
        if (target) {
          target.scrollIntoView({ behavior: "smooth" });
        }
        history.replaceState(null, "", link.href);
        return;
      }

      // Skip links outside panes (header, footer, etc.) but allow sidebar
      if (
        !link.closest(CONFIG.selectors.pane) &&
        !link.closest(CONFIG.selectors.siteNav)
      )
        return;

      // Check if internal link
      const href = link.getAttribute("href");
      // Modify to check if href is the same domain
      const isInternal =
        href &&
        link.hostname === window.location.hostname &&
        !href.startsWith("mailto:") &&
        !href.startsWith("#") &&
        !link.hasAttribute("download");

      if (!isInternal) return;

      event.preventDefault();

      // Check if it's a sidebar link
      const isSidebarLink = link.closest(CONFIG.selectors.siteNav);

      if (isSidebarLink) {
        // Clear all panes for sidebar navigation
        this.paneSystem.panesContainer.innerHTML = "";
        // Only reset wide-view if manual toggle is off
        if (!this.paneSystem.mediaQueryHandler.isWideView) {
          document.body.classList.remove(CONFIG.classes.wideView);
          document.body.classList.remove(CONFIG.classes.hasTransition);
        }
      }

      // Fetch and append new pane
      this.fetchAndAppendPane(link.href, isSidebarLink, event).catch(
        console.error,
      );
    }

    async fetchAndAppendPane(href, isSidebarLink, event) {
      try {
        const response = await fetch(href);
        if (!response.ok) {
          throw new Error(`Failed to fetch ${href}: ${response.status}`);
        }
        const html = await response.text();

        // For content links, find current pane and remove subsequent ones
        if (!isSidebarLink && event && event.target) {
          const currentPane = event.target.closest(CONFIG.selectors.pane);
          if (!currentPane) return;

          const panes = Array.from(this.paneSystem.panesContainer.children);
          const currentIndex = panes.indexOf(currentPane);

          // Remove panes after current
          panes.slice(currentIndex + 1).forEach((pane) => pane.remove());
        }

        const parser = new DOMParser();
        const doc = parser.parseFromString(html, "text/html");
        const mainContent = Utils.safeQuerySelector(
          CONFIG.selectors.mainContent,
          doc,
        );
        if (!mainContent) return;

        // Extract title from content
        const titleElement = mainContent.querySelector(
          CONFIG.selectors.titleElements,
        );
        const titleText = titleElement
          ? titleElement.textContent.trim()
          : "Untitled";

        // Create new pane with spine and body
        const newPane = document.createElement("div");
        newPane.className = "pane";
        newPane.innerHTML = `
        <div class="pane-content">
          <div class="pane-header">${titleText}</div>
          <div class="pane-body">
            ${mainContent.outerHTML}
          </div>
        </div>
      `;

        this.paneSystem.panesContainer.appendChild(newPane);
        newPane.dataset.url = href;

        // Add copy buttons to the new pane
        Utils.addCodeCopyButtons();

        // Check if panes overflow and enable wide view (only if manual toggle is off)
        this.paneSystem.mediaQueryHandler.checkOverflowAndEnableWideView();

        // Re-initialize Mermaid diagrams in the new pane
        const mermaidElements = Utils.safeQuerySelectorAll(
          CONFIG.selectors.mermaidElements,
          newPane,
        );
        mermaidElements.forEach((el) => Utils.renderMermaid(el));

        // Scroll to new pane
        newPane.scrollIntoView({
          behavior: "smooth",
          block: "nearest",
          inline: "end",
        });

        // Update layout after adding pane
        setTimeout(() => {
          this.paneSystem.layoutManager.updateLayout();
          this.paneSystem.layoutManager.addPaneHeaderClickHandlers();
        }, CONFIG.timeouts.layoutUpdate);

        // Update URL
        history.pushState(null, "", href);
        // Update active nav item
        Utils.updateActiveNav(href);
      } catch (error) {
        console.error("Error fetching and appending pane:", error);
      }
    }

    loadPaneForUrl(href) {
      this.fetchAndAppendPane(href, false, null).catch(console.error);
    }
  }

  class LayoutManager {
    constructor(paneSystem) {
      this.paneSystem = paneSystem;
    }

    addPaneHeaderClickHandlers() {
      const panes = Utils.safeQuerySelectorAll(CONFIG.selectors.pane);
      panes.forEach((pane) => {
        const paneHeader = Utils.safeQuerySelector(
          CONFIG.selectors.paneHeader,
          pane,
        );
        if (paneHeader && !paneHeader.hasClickHandler) {
          paneHeader.addEventListener("click", () =>
            this.handlePaneHeaderClick(pane),
          );
          paneHeader.hasClickHandler = true; // Prevent duplicate handlers
        }
      });
    }

    handlePaneHeaderClick(pane) {
      const indexVar = parseFloat(
        getComputedStyle(pane).getPropertyValue("--index"),
      );
      const numCardsVar = parseFloat(
        getComputedStyle(pane).getPropertyValue("--numcards"),
      );

      if (!isNaN(indexVar) && !isNaN(numCardsVar) && numCardsVar > 0) {
        const scrollPercentage = indexVar / numCardsVar;
        const panesContainer = pane.closest(CONFIG.selectors.panesContainer);

        if (panesContainer) {
          const scrollLeft =
            scrollPercentage *
            (panesContainer.scrollWidth - panesContainer.clientWidth);
          panesContainer.scrollTo({
            left: scrollLeft,
            behavior: "smooth",
          });
        }
      }
    }

    updateLayout() {
      if (!this.paneSystem.enabled) return;
      const container = this.paneSystem.panesContainer;
      const scroller = Utils.safeQuerySelector(CONFIG.selectors.mainScroller); // assuming .main is the scroller
      const panes = Utils.safeQuerySelectorAll(
        CONFIG.selectors.pane,
        container,
      );

      const style = window.getComputedStyle(document.body);

      // Configuration
      const spineWidth = Number(
        style.getPropertyValue("--spine-width").replace("px", ""),
      );
      const defaultCardWidth = Number(
        style.getPropertyValue("--card-width").replace("px", ""),
      );
      const paddingLeft = CONFIG.widths.paddingLeft; // 2rem
      const paddingRight = CONFIG.widths.paddingRight; // 2rem buffer
      const numCards = panes.length;

      // 1. Set Basics
      container.style.setProperty("--numcards", numCards);
      panes.forEach((pane, index) => {
        pane.style.setProperty("--index", index + 1);
      });

      // 2. Flexible Last Card Calculation
      const viewportWidth = scroller ? scroller.clientWidth : window.innerWidth;

      // Space taken by spines of all previous cards
      // (numCards - 1) cards are stacked on the left
      const spinesTotalWidth = (numCards - 1) * spineWidth;

      // Calculate remaining space for the last card
      // Viewport - (Left Padding + Spines) - Right Padding Buffer
      let availableWidth =
        viewportWidth - (paddingLeft + spinesTotalWidth) - paddingRight;

      // Logic: Shrink last card if needed, respect minimum
      const minWidth = CONFIG.widths.minCardWidth;
      let finalWidth = defaultCardWidth;

      if (availableWidth < defaultCardWidth) {
        finalWidth = Math.max(minWidth, availableWidth);
      }

      // Apply the width
      container.style.setProperty("--last-card-width", `${finalWidth}px`);
    }
  }

  class HistoryManager {
    constructor(paneSystem) {
      this.paneSystem = paneSystem;
      this.initialUrl = window.location.href;
      this.enabled = false;
      this.initialized = false;
    }

    init() {
      if (this.initialized) return;
      window.addEventListener("popstate", () => this.handlePopState());
      this.initialized = true;
    }

    handlePopState() {
      if (!this.enabled) return;
      const panes = this.paneSystem.panesContainer.children;
      const currentHref = window.location.href;
      const paneUrls = Array.from(panes).map((pane) => pane.dataset.url);

      const matchingIndex = paneUrls.indexOf(currentHref);

      if (matchingIndex >= 0) {
        // Remove panes after the matching one
        for (let i = panes.length - 1; i > matchingIndex; i--) {
          panes[i].remove();
        }
        this.updateAfterChange(currentHref);
      } else {
        // Clear existing panes and load pane for current URL
        this.paneSystem.panesContainer.innerHTML = "";
        this.paneSystem.linkInterceptor.loadPaneForUrl(currentHref);
      }
    }

    updateAfterChange(currentHref) {
      // Update active nav item
      Utils.updateActiveNav(currentHref);
      // Check if still overflow after removing (only if manual toggle is off)
      this.paneSystem.mediaQueryHandler.removeWideViewIfNoOverflow();
      // Update layout
      setTimeout(() => {
        this.paneSystem.layoutManager.updateLayout();
        this.paneSystem.layoutManager.addPaneHeaderClickHandlers();
      }, CONFIG.timeouts.layoutUpdate);
    }
  }

  // Main PaneSystem class
  class PaneSystem {
    constructor() {
      this.panesContainer = Utils.safeQuerySelector(
        CONFIG.selectors.panesContainer,
      );
      if (!this.panesContainer) {
        console.error(
          "Panes container not found. Pane system cannot initialize.",
        );
        return;
      }
      this.enabled = false;
      this.mediaQueryHandler = new MediaQueryHandler(this);
      this.linkInterceptor = new LinkInterceptor(this);
      this.layoutManager = new LayoutManager(this);
      this.historyManager = new HistoryManager(this);
      this.mqMd = window.matchMedia(CONFIG.mediaQuery);
    }

    init() {
      // Enable transitions after initial layout
      setTimeout(() => {
        document.body.classList.add(CONFIG.classes.hasTransition);
      }, CONFIG.timeouts.initialLayout);

      // Update active nav for initial page
      setTimeout(() => Utils.updateActiveNav(window.location.href), 100);

      // Initialize components
      this.mediaQueryHandler.init();

      // Listen for changes
      this.mqMd.addEventListener("change", () =>
        this.mediaQueryHandler.onMediaQueryChange(),
      );
    }

    enablePaneSystem() {
      if (this.enabled) return;
      this.enabled = true;
      this.linkInterceptor.enabled = true;
      this.historyManager.enabled = true;
      this.linkInterceptor.init();
      this.historyManager.init();
      this.layoutManager.updateLayout();
      this.layoutManager.addPaneHeaderClickHandlers();
    }

    disablePaneSystem() {
      if (!this.enabled) return;
      this.enabled = false;
      this.linkInterceptor.enabled = false;
      this.historyManager.enabled = false;
    }
  }
  // Initialize the pane system
  const paneSystem = new PaneSystem();
  paneSystem.init();
});
