# Quickstart: Running Workspace Puppeteer Tests

This guide walks you through executing and verifying the Puppeteer automated browser testing suite for the Oxibase Workspace.

## 1. Prerequisites

Ensure you have Node.js (v18+) and npm installed locally on your system.

## 2. Setup

Install Puppeteer (which automatically downloads the compatible Chromium browser binary) inside your development workspace directory:

```bash
npm install puppeteer --no-save
```

## 3. Running the Tests

To launch the self-contained testing script, which automatically compiles and boots the database server, runs all interactive browser assertions, and shuts down clean:

```bash
node specs/054-workspace-puppeteer-tests/test-workspace.js
```

### Headful / Debug Mode
To watch the browser window open and interact with the UI live during testing, set the `HEADLESS` environment variable to `false`:

```bash
HEADLESS=false node specs/054-workspace-puppeteer-tests/test-workspace.js
```
