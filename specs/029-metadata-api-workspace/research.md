# Research: DaisyUI Integration

## Needs Clarification
- **How to integrate DaisyUI via CDN?**
  DaisyUI v5 can be used directly from CDN without installation. It requires adding a CSS link and the Tailwind browser script:
  ```html
  <link href="https://cdn.jsdelivr.net/npm/daisyui@5" rel="stylesheet" type="text/css" />
  <script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4"></script>
  ```
  By default, `light` and `dark` themes are included.

## Technology Choices

### Decision: DaisyUI over Raw TailwindCSS
- **Decision**: The Workspace application will use DaisyUI component classes instead of building UI elements purely from raw TailwindCSS utility classes.
- **Rationale**: DaisyUI provides semantic component class names (e.g., `btn`, `card`, `modal`), drastically reducing HTML verbosity while retaining the flexibility of Tailwind. This aligns with the project's goal of an "EXTREMELY thin" frontend and makes the server-rendered Minijinja templates much easier to read and maintain. User explicitly requested this change.
- **Alternatives considered**: Raw TailwindCSS (initial spec assumption, rejected in favor of DaisyUI for better maintainability), Bootstrap, Bulma.

### Decision: Unpoly for Frontend Interactions
- **Decision**: Use Unpoly (via CDN) to handle client-side interactions (modals, partial page updates, navigation).
- **Rationale**: As specified in the requirements, Unpoly allows the application to feel like an SPA while keeping all rendering logic on the server via Minijinja templates, satisfying the requirement for an "EXTREMELY thin layer of JS".
- **Alternatives considered**: Vue.js, React, HTMX. Unpoly was specifically requested and fits the server-driven HTML paradigm perfectly.