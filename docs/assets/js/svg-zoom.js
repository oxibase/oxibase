/**
 * Universal SVG Pan & Zoom Controller
 * * Drop-in script that automatically adds:
 * 1. Wheel Zoom (zooms towards mouse pointer)
 * 2. Click & Drag Panning
 * 3. Smart Click Handling (ignores clicks if user was dragging)
 * 4. Hover "Expand" button -> Opens 80% screen modal with blur backdrop
 * * Works on all <svg> elements in the document via Event Delegation.
 */

document.addEventListener("DOMContentLoaded", function () {
  "use strict";

  // --- Configuration ---
  const CONFIG = {
    zoomSpeed: 1.06,
    maxZoomWidth: 10000,
    minZoomWidth: 10,
    dragThreshold: 5,
    // The icon provided by the user
    expandIconSvg: `<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" viewBox="0 0 256 256"><path d="M216,48V96a8,8,0,0,1-16,0V67.31l-42.34,42.35a8,8,0,0,1-11.32-11.32L188.69,56H160a8,8,0,0,1,0-16h48A8,8,0,0,1,216,48ZM98.34,146.34,56,188.69V160a8,8,0,0,0-16,0v48a8,8,0,0,0,8,8H96a8,8,0,0,0,0-16H67.31l42.35-42.34a8,8,0,0,0-11.32-11.32ZM208,152a8,8,0,0,0-8,8v28.69l-42.34-42.35a8,8,0,0,0-11.32-11.32L188.69,200H160a8,8,0,0,0,0,16h48a8,8,0,0,0,8-8V160A8,8,0,0,0,208,152ZM67.31,56H96a8,8,0,0,0,0-16H48a8,8,0,0,0-8,8V96a8,8,0,0,0,16,0V67.31l42.34,42.35a8,8,0,0,0,11.32-11.32Z"></path></svg>`,
    resetIconSvg: `<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" viewBox="0 0 32 32"><path d="M18,28A12,12,0,1,0,6,16v6.2L2.4,18.6,1,20l6,6,6-6-1.4-1.4L8,22.2V16H8A10,10,0,1,1,18,26Z"/></svg>`,
  };

  // --- Global State ---
  let panState = {
    isDragging: false,
    hasMoved: false,
    svg: null,
    startPoint: { x: 0, y: 0, w: 0, h: 0 },
    startViewBox: { x: 0, y: 0, w: 0, h: 0 },
  };

  // UI State
  let uiState = {
    activeSvg: null, // The SVG currently being hovered
    expandBtn: null,
    resetBtn: null,
    modalOverlay: null,
    modalContent: null,
  };

  /**
   * Helper: Ensure we are interacting with an SVG element
   * FIXED: Explicitly ignores the SVG icon inside our own Expand Button
   */
  const getTargetSvg = (e) => {
    const svg = e.target.closest("svg");
    // If no SVG, or if the SVG is part of our UI button, or not a mermaid SVG, ignore it
    if (
      !svg ||
      svg.closest(".svg-pan-expand-btn") ||
      svg.closest(".svg-pan-reset-btn") ||
      !svg.id.startsWith("mermaid-")
    )
      return null;
    return svg;
  };

  // =========================================================
  // SECTION 1: UI Injection (Button, Modal)
  // =========================================================

  const createUI = () => {
    // 1. Expand Button
    const btn = document.createElement("div");
    btn.className = "svg-pan-expand-btn";
    btn.innerHTML = CONFIG.expandIconSvg;
    btn.title = "Expand View";
    document.body.appendChild(btn);
    uiState.expandBtn = btn;

    // 1.5. Reset Button
    const resetBtn = document.createElement("div");
    resetBtn.className = "svg-pan-reset-btn";
    resetBtn.innerHTML = CONFIG.resetIconSvg;
    resetBtn.title = "Reset Zoom & Pan";
    document.body.appendChild(resetBtn);
    uiState.resetBtn = resetBtn;

    // 2. Modal Overlay
    const overlay = document.createElement("div");
    overlay.className = "svg-pan-modal-overlay";

    // 3. Modal Content Container
    const content = document.createElement("div");
    content.className = "svg-pan-modal-content";

    // 4. Close Button
    const closeBtn = document.createElement("button");
    closeBtn.className = "svg-pan-modal-close";
    closeBtn.innerHTML = "&times;";
    closeBtn.onclick = closeModal;

    // Assembly
    content.appendChild(closeBtn);
    overlay.appendChild(content);
    document.body.appendChild(overlay);

    uiState.modalOverlay = overlay;
    uiState.modalContent = content;

    // Events
    btn.addEventListener("click", openModal);
    resetBtn.addEventListener("click", resetZoom);
    overlay.addEventListener("click", (e) => {
      if (e.target === overlay) closeModal();
    });
  };

  const updateButtonPosition = () => {
    if (!uiState.activeSvg || !uiState.expandBtn || !uiState.resetBtn) return;

    const rect = uiState.activeSvg.getBoundingClientRect();
    const btnSize = 34; // approx size with padding
    const gap = 5;

    // Position top-right corner of SVG, accounting for scroll
    const top = rect.top + window.scrollY + 10;
    const expandLeft = rect.left + rect.width + window.scrollX - btnSize - 10;
    const resetLeft = expandLeft - btnSize - gap;

    uiState.expandBtn.style.top = `${top}px`;
    uiState.expandBtn.style.left = `${expandLeft}px`;
    uiState.expandBtn.style.display = "block";

    uiState.resetBtn.style.top = `${top}px`;
    uiState.resetBtn.style.left = `${resetLeft}px`;
    uiState.resetBtn.style.display = "block";
  };

  const openModal = () => {
    if (!uiState.activeSvg) return;

    // Clone the SVG
    const clone = uiState.activeSvg.cloneNode(true);
    // Copy the original viewbox data attribute
    if (uiState.activeSvg.dataset.originalViewbox) {
      clone.dataset.originalViewbox = uiState.activeSvg.dataset.originalViewbox;
    }

    // Ensure clone fills container (remove fixed w/h if present)
    clone.removeAttribute("width");
    clone.removeAttribute("height");
    clone.style.width = "100%";
    clone.style.height = "100%";
    clone.style.removeProperty("max-width");
    clone.style.cursor = "grab"; // Hint that it's pannable

    // Clean previous content (keep close button)
    // We select the close button to make sure we don't delete it
    const closeBtn = uiState.modalContent.querySelector(".svg-pan-modal-close");
    uiState.modalContent.innerHTML = "";
    uiState.modalContent.appendChild(closeBtn);

    // Create and add the reset button for the modal
    const modalResetBtn = document.createElement("button");
    modalResetBtn.className = "svg-pan-modal-reset";
    modalResetBtn.innerHTML = CONFIG.resetIconSvg;
    modalResetBtn.title = "Reset Zoom & Pan";
    modalResetBtn.onclick = () => {
      const originalViewBox = clone.dataset.originalViewbox;
      if (originalViewBox) {
        clone.setAttribute("viewBox", originalViewBox);
      }
    };
    uiState.modalContent.appendChild(modalResetBtn);

    uiState.modalContent.appendChild(clone);

    // Show Modal
    uiState.modalOverlay.classList.add("active");
    document.body.style.overflow = "hidden"; // Disable background scroll
  };

  const closeModal = () => {
    uiState.modalOverlay.classList.remove("active");
    document.body.style.overflow = "";

    // Wait for transition then clear content to save memory
    setTimeout(() => {
      if (!uiState.modalOverlay.classList.contains("active")) {
        const closeBtn = uiState.modalContent.querySelector(
          ".svg-pan-modal-close",
        );
        // Also find the reset button if it exists
        const resetBtn = uiState.modalContent.querySelector(
          ".svg-pan-modal-reset",
        );
        uiState.modalContent.innerHTML = "";
        if (closeBtn) uiState.modalContent.appendChild(closeBtn);
        if (resetBtn) uiState.modalContent.appendChild(resetBtn);
      }
    }, 300);
  };

  const resetZoom = () => {
    if (!uiState.activeSvg) return;
    const originalViewBox = uiState.activeSvg.dataset.originalViewbox;
    if (originalViewBox) {
      uiState.activeSvg.setAttribute("viewBox", originalViewBox);
    }
  };

  // =========================================================
  // SECTION 2: Event Listeners (Logic)
  // =========================================================

  // Initialize UI
  createUI();

  // Hover Logic: Show Expand Button
  document.addEventListener("mousemove", (e) => {
    // If dragging, don't mess with UI
    if (panState.isDragging) return;

    // DEBUG: Uncomment the line below to see what the script detects under your mouse
    // console.log("Hovering:", e.target, "Detected SVG:", getTargetSvg(e));

    const svg = getTargetSvg(e);
    const overBtn = e.target.closest(".svg-pan-expand-btn, .svg-pan-reset-btn");

    // Case 1: Mouse is over an actual SVG (not our button icon)
    if (svg) {
      if (
        uiState.activeSvg !== svg ||
        uiState.expandBtn.style.display === "none"
      ) {
        uiState.activeSvg = svg;
        // Store original viewbox if we haven't already
        if (!svg.dataset.originalViewbox) {
          const viewBox = svg.getAttribute("viewBox");
          if (viewBox) {
            svg.dataset.originalViewbox = viewBox;
          }
        }
        updateButtonPosition();
      }
    }
    // Case 2: Mouse is over the Expand Button
    else if (overBtn) {
      // Do nothing, just keep the button visible.
      // (Previously this was detecting the Icon's SVG and causing the loop)
    }
    // Case 3: Mouse is over nothing relevant
    else {
      if (uiState.activeSvg) {
        uiState.expandBtn.style.display = "none";
        if (uiState.resetBtn) uiState.resetBtn.style.display = "none";
        uiState.activeSvg = null;
      }
    }
  });

  // Update button pos on scroll or resize (so it doesn't float away or detach)
  const updatePos = () => {
    if (uiState.activeSvg) updateButtonPosition();
  };
  document.addEventListener("scroll", updatePos, {
    capture: true,
    passive: true,
  });
  window.addEventListener("resize", updatePos, { passive: true });

  // =========================================================
  // SECTION 3: Pan & Zoom Logic (Existing)
  // =========================================================

  document.addEventListener(
    "wheel",
    (e) => {
      const svg = getTargetSvg(e);
      if (!svg) return;

      e.preventDefault();

      const viewBox = svg.viewBox.baseVal;
      const rect = svg.getBoundingClientRect();

      const mouseX = e.clientX - rect.left;
      const mouseY = e.clientY - rect.top;

      const zoomFactor = Math.pow(CONFIG.zoomSpeed, e.deltaY > 0 ? 1 : -1);

      const vbWidth = viewBox.width;
      const vbHeight = viewBox.height;

      let newWidth = vbWidth * zoomFactor;
      let newHeight = vbHeight * zoomFactor;

      newWidth = Math.max(
        CONFIG.minZoomWidth,
        Math.min(CONFIG.maxZoomWidth, newWidth),
      );
      newHeight = Math.max(
        (CONFIG.minZoomWidth / vbWidth) * vbHeight,
        Math.min((CONFIG.maxZoomWidth / vbWidth) * vbHeight, newHeight),
      );

      const mouseXInSvg = viewBox.x + (mouseX / rect.width) * vbWidth;
      const mouseYInSvg = viewBox.y + (mouseY / rect.height) * vbHeight;

      const newX = mouseXInSvg - (mouseX / rect.width) * newWidth;
      const newY = mouseYInSvg - (mouseY / rect.height) * newHeight;

      svg.setAttribute("viewBox", `${newX} ${newY} ${newWidth} ${newHeight}`);
    },
    { passive: false },
  );

  document.addEventListener("pointerdown", (e) => {
    if (e.button !== 0) return;
    const svg = getTargetSvg(e);
    if (!svg) return;

    // Don't trigger drag if clicking the expand button
    if (
      e.target.closest(".svg-pan-expand-btn") ||
      e.target.closest(".svg-pan-reset-btn")
    )
      return;

    e.preventDefault();

    const rect = svg.getBoundingClientRect();
    const viewBox = svg.viewBox.baseVal;

    panState = {
      isDragging: true,
      hasMoved: false,
      svg: svg,
      startPoint: { x: e.clientX, y: e.clientY, w: rect.width, h: rect.height },
      startViewBox: {
        x: viewBox.x,
        y: viewBox.y,
        w: viewBox.width,
        h: viewBox.height,
      },
    };

    svg.style.cursor = "grabbing";
    svg.setPointerCapture(e.pointerId);
  });

  document.addEventListener("pointermove", (e) => {
    if (!panState.isDragging || !panState.svg) return;

    e.preventDefault();

    const dx = e.clientX - panState.startPoint.x;
    const dy = e.clientY - panState.startPoint.y;

    if (Math.sqrt(dx * dx + dy * dy) > CONFIG.dragThreshold) {
      panState.hasMoved = true;
    }

    const scaleX = panState.startViewBox.w / panState.startPoint.w;
    const scaleY = panState.startViewBox.h / panState.startPoint.h;

    const newX = panState.startViewBox.x - dx * scaleX;
    const newY = panState.startViewBox.y - dy * scaleY;

    panState.svg.setAttribute(
      "viewBox",
      `${newX} ${newY} ${panState.startViewBox.w} ${panState.startViewBox.h}`,
    );
  });

  const stopDrag = (e) => {
    if (!panState.isDragging || !panState.svg) return;

    panState.isDragging = false;
    panState.svg.style.cursor = "";

    if (panState.svg.hasPointerCapture(e.pointerId)) {
      panState.svg.releasePointerCapture(e.pointerId);
    }
  };

  document.addEventListener("pointerup", stopDrag);
  document.addEventListener("pointercancel", stopDrag);

  document.addEventListener(
    "click",
    (e) => {
      if (panState.hasMoved) {
        e.preventDefault();
        e.stopPropagation();
        panState.hasMoved = false;
      }
    },
    { capture: true },
  );
});
