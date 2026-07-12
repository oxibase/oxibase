# Research: Workspace Puppeteer Testing Plan

This document records the technical context, design choices, and alternative evaluations for orchestrating Puppeteer-based end-to-end tests for the Oxibase Workspace.

---

## 1. Testing Library Wrapper & Assertions

### Decision
Use a lightweight, self-contained, native Node.js testing script (`test-workspace.js`) utilizing Node's built-in `node:assert` and `node:test` (available in Node.js v18+) instead of installing heavy external frameworks like Jest or Mocha.

### Rationale
Adhering to the minimal and modern mainframe monolith guidelines, a self-contained vanilla Node script avoids unnecessary external packages. It requires zero configuration files, runs instantly with single-digit millisecond startup overhead, and provides modern async test runners out of the box.

### Alternatives Considered
- **Jest**: Highly feature-rich, but pulls in hundreds of transitive dependencies, slowing down local install times and bloating the workspace environment.
- **Mocha & Chai**: Lightweight but still requires global installation or extra dependencies. Built-in `node:test` + `node:assert` achieves the exact same result with zero dependencies.

---

## 2. Server Lifecycle & Orchestration

### Decision
The test runner script will automatically spawn the `oxibase serve` process on a designated test port (defaulting to `8080`), poll the port until it is active, run the Puppeteer suite, and reliably kill the database process using process signals (`SIGINT` or `SIGKILL`) on exit.

### Rationale
Ensures that tests are completely self-contained and reproducible, running successfully in CI/CD pipelines without expecting an active pre-running server instance.

### Alternatives Considered
- **Pre-running server expectation**: Fails under automated CI/CD runs where the server isn't already bound. Spawning dynamically guarantees success.
- **Docker Compose Orchestration**: Adds infrastructure complexity and goes against the mainframe monolith principle (infrastructure is data / self-contained binary). Spawning the native compiled binary is faster and zero-overhead.

---

## 3. Selector Anchors & Flakiness Prevention

### Decision
Identify elements using robust CSS selectors, standard HTML roles, and explicit `page.waitForSelector()` waits to handle dynamic and asynchronous AJAX database loading transitions (e.g. Gantt trace trees, logs lists) without resorting to fragile hardcoded sleeps.

### Rationale
Workspace logs and traces load dynamically via dynamic AJAX Fetch calls. Native DOM wait transitions prevent race conditions and eliminate flaky test results.
