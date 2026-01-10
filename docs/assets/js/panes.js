document.addEventListener("DOMContentLoaded", function () {
  const panesContainer = document.querySelector(".panes-container");

  // Check if media query mq(md) is active (min-width: 50rem based on SCSS)
  const mqMd = window.matchMedia("(min-width: 50rem)");
  let isWideView = false;

  function initPanes() {
    if (mqMd.matches) {
      // Enable transitions after initial layout
      setTimeout(() => {
        document.body.classList.add("has-transition");
        updateLayout();
        addPaneHeaderClickHandlers();
      }, 500);

      // Update layout on resize
      window.addEventListener("resize", updateLayout);

      // Wide view toggle
      const wideViewToggle = document.getElementById("wide-view-toggle");
      if (wideViewToggle) {
        wideViewToggle.style.display = ""; // Reset to CSS default
        // Load saved state (per session)
        isWideView = sessionStorage.getItem("wide-view") === "true";
        wideViewToggle.checked = isWideView;
        if (isWideView) {
          document.body.classList.add("wide-view");
        }

        // Handle toggle
        wideViewToggle.addEventListener("change", function () {
          const enabled = this.checked;
          sessionStorage.setItem("wide-view", enabled);
          isWideView = enabled;
          if (enabled) {
            document.body.classList.add("wide-view");
          } else {
            // When disabling manual, apply automatic if overflow
            if (panesContainer.scrollWidth > panesContainer.clientWidth) {
              document.body.classList.add("wide-view");
            } else {
              document.body.classList.remove("wide-view");
            }
          }
        });
      }

      // Intercept all internal links
      document.addEventListener("click", function (e) {
        const link = e.target.closest("a");
        if (!link) return;

        // Skip links outside panes (header, footer, etc.) but allow sidebar
        if (!link.closest(".pane") && !link.closest(".site-nav")) return;

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

        e.preventDefault();

        // Check if it's a sidebar link
        const isSidebarLink = link.closest(".site-nav");

        if (isSidebarLink) {
          // Clear all panes for sidebar navigation
          panesContainer.innerHTML = "";
          // Only reset wide-view if manual toggle is off
          if (!isWideView) {
            document.body.classList.remove("wide-view");
            document.body.classList.remove("has-transition");
          }
        }

        // Fetch and append new pane
        fetch(link.href)
          .then((response) => response.text())
          .then((html) => {
            // For content links, find current pane and remove subsequent ones
            if (!isSidebarLink) {
              const currentPane = e.target.closest(".pane");
              if (!currentPane) return;

              const panes = Array.from(panesContainer.children);
              const currentIndex = panes.indexOf(currentPane);

              // Remove panes after current
              panes.slice(currentIndex + 1).forEach((pane) => pane.remove());
            }

            const parser = new DOMParser();
            const doc = parser.parseFromString(html, "text/html");
            const mainContent = doc.getElementById("main-content");
            if (!mainContent) return;

            // Extract title from content
            const titleElement = mainContent.querySelector("h1, h2");
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

            panesContainer.appendChild(newPane);

            // Check if panes overflow and enable wide view (only if manual toggle is off)
            if (
              !isWideView &&
              panesContainer.scrollWidth > panesContainer.clientWidth
            ) {
              document.body.classList.add("wide-view");
              document.body.classList.add("has-transition");
            }

            // Re-initialize Mermaid diagrams in the new pane
            if (window.mermaid && typeof window.mermaid.render === "function") {
              const mermaidElements =
                newPane.querySelectorAll(".language-mermaid");
              mermaidElements.forEach(async (el) => {
                try {
                  const id =
                    "mermaid-" + Math.random().toString(36).substr(2, 9);
                  const { svg } = await window.mermaid.render(
                    id,
                    el.textContent.trim(),
                  );
                  // Replace the <pre><code> with the SVG
                  el.parentElement.outerHTML = svg;
                } catch (error) {
                  console.error("Mermaid render error:", error);
                }
              });
            }

            // Scroll to new pane
            newPane.scrollIntoView({
              behavior: "smooth",
              block: "nearest",
              inline: "end",
            });

            // Update layout after adding pane
            setTimeout(() => {
              updateLayout();
              addPaneHeaderClickHandlers();
            }, 50);

            // Update URL
            history.pushState(null, "", link.href);
          })
          .catch(console.error);
      });

      // Handle browser back/forward
      window.addEventListener("popstate", function () {
        const panes = panesContainer.children;
        if (panes.length > 1) {
          panes[panes.length - 1].remove();
          // Check if still overflow after removing (only if manual toggle is off)
          if (
            !isWideView &&
            panesContainer.scrollWidth <= panesContainer.clientWidth
          ) {
            document.body.classList.remove("wide-view");
            document.body.classList.remove("has-transition");
          }
          // Update layout
          setTimeout(() => {
            updateLayout();
            addPaneHeaderClickHandlers();
          }, 50);
        }
      });
    } else {
      // Disable on small screens - remove classes and reset
      document.body.classList.remove("has-transition", "wide-view");
      // Reset last card width
      panesContainer.style.setProperty(
        "--last-card-width",
        "var(--card-width)",
      );
      // Hide toggle
      const wideViewToggle = document.getElementById("wide-view-toggle");
      if (wideViewToggle) {
        wideViewToggle.style.display = "none";
      }
    }
  }

  // Initial check
  initPanes();

  // Listen for changes
  mqMd.addEventListener("change", initPanes);

  // Add click handlers to all pane headers
  function addPaneHeaderClickHandlers() {
    const panes = document.querySelectorAll(".pane");
    panes.forEach((pane) => {
      const paneHeader = pane.querySelector(".pane-header");
      if (paneHeader && !paneHeader.hasClickHandler) {
        paneHeader.addEventListener("click", () => {
          const indexVar = parseFloat(
            getComputedStyle(pane).getPropertyValue("--index"),
          );
          const numCardsVar = parseFloat(
            getComputedStyle(pane).getPropertyValue("--numcards"),
          );

          if (!isNaN(indexVar) && !isNaN(numCardsVar) && numCardsVar > 0) {
            const scrollPercentage = indexVar / numCardsVar;
            const panesContainer = pane.closest(".panes-container");

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
        });
        paneHeader.hasClickHandler = true; // Prevent duplicate handlers
      }
    });
  }

  // Layout update for flexible last card
  function updateLayout() {
    const container = panesContainer;
    const scroller = document.querySelector(".main"); // assuming .main is the scroller
    const panes = container.querySelectorAll(".pane");

    var style = window.getComputedStyle(document.body);

    // Configuration
    const spineWidth = Number(
      style.getPropertyValue("--spine-width").replace("px", ""),
    );
    const defaultCardWidth = Number(
      style.getPropertyValue("--card-width").replace("px", ""),
    );
    const paddingLeft = 32; // 2rem
    const paddingRight = 32; // 2rem buffer
    const numCards = panes.length;

    // 1. Set Basics
    container.style.setProperty("--numcards", numCards);
    panes.forEach((pane, index) => {
      pane.style.setProperty("--index", index + 1);
    });

    // 2. Flexible Last Card Calculation
    const viewportWidth = scroller.clientWidth;

    // Space taken by spines of all previous cards
    // (numCards - 1) cards are stacked on the left
    const spinesTotalWidth = (numCards - 1) * spineWidth;

    // Calculate remaining space for the last card
    // Viewport - (Left Padding + Spines) - Right Padding Buffer
    let availableWidth =
      viewportWidth - (paddingLeft + spinesTotalWidth) - paddingRight;

    // Logic: Shrink last card if needed, respect minimum
    const minWidth = 350;
    let finalWidth = defaultCardWidth;

    if (availableWidth < defaultCardWidth) {
      finalWidth = Math.max(minWidth, availableWidth);
    }

    // Apply the width
    container.style.setProperty("--last-card-width", `${finalWidth}px`);
  }
});
