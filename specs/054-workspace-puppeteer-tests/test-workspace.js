// Copyright 2026 Oxibase Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

const { spawn } = require('child_process');
const http = require('http');
const path = require('path');
const fs = require('fs');
const test = require('node:test');
const assert = require('node:assert');
const puppeteer = require('puppeteer');

const PORT = process.env.PORT || 8080;
const HEADLESS = process.env.HEADLESS !== 'false';

function cleanupDb() {
  const dbFiles = [
    'workspace_test.db',
    'workspace_test.db-wal',
    'workspace_test.db-shm',
    'workspace_test.db.wal',
    'workspace_test.db.shm'
  ];
  dbFiles.forEach(file => {
    try {
      const p = path.join(process.cwd(), file);
      if (fs.existsSync(p)) {
        fs.unlinkSync(p);
      }
    } catch (e) {
      // ignore
    }
  });
}

const runInstallWorkspace = () => {
  return new Promise((resolve, reject) => {
    const dbPath = path.join(process.cwd(), 'workspace_test.db');
    console.log(`Running install-workspace on ${dbPath}...`);
    const proc = spawn('./target/debug/oxibase', ['install-workspace', '-d', `file://${dbPath}`], {
      cwd: process.cwd()
    });
    proc.on('close', (code) => {
      if (code === 0) {
        console.log('Workspace installed successfully.');
        resolve();
      } else {
        reject(new Error(`install-workspace exited with code ${code}`));
      }
    });
    proc.on('error', reject);
  });
};

let serveProcess = null;
const runServe = () => {
  return new Promise((resolve, reject) => {
    const dbPath = path.join(process.cwd(), 'workspace_test.db');
    console.log(`Starting oxibase serve on port ${PORT}...`);
    serveProcess = spawn('./target/debug/oxibase', ['serve', '-d', `file://${dbPath}`, '-p', String(PORT)], {
      cwd: process.cwd()
    });
    
    serveProcess.stdout.on('data', (data) => console.log(`SERVER STDOUT: ${data.toString().trim()}`));
    serveProcess.stderr.on('data', (data) => console.log(`SERVER STDERR: ${data.toString().trim()}`));

    serveProcess.on('error', (err) => {
      reject(err);
    });
    
    serveProcess.on('close', (code) => {
      console.log(`Serve process exited with code ${code}`);
    });

    const startTime = Date.now();
    const interval = setInterval(() => {
      const req = http.get(`http://127.0.0.1:${PORT}/workspace`, (res) => {
        if (res.statusCode === 200 || res.statusCode === 302) {
          clearInterval(interval);
          console.log(`Server is up and running on port ${PORT}.`);
          resolve();
        }
      });
      req.on('error', () => {
        if (Date.now() - startTime > 15000) { // 15s timeout
          clearInterval(interval);
          reject(new Error('Timeout waiting for server to bind port 8080'));
        }
      });
    }, 250);
  });
};

function killServeProcess() {
  if (serveProcess) {
    console.log('Stopping oxibase serve process...');
    try {
      serveProcess.kill('SIGINT');
    } catch (e) {
      // ignore
    }
    setTimeout(() => {
      try {
        serveProcess.kill('SIGKILL');
      } catch (e) {
        // ignore
      }
    }, 1000).unref();
  }
}

process.on('exit', () => {
  killServeProcess();
  cleanupDb();
});

process.on('SIGINT', () => {
  killServeProcess();
  cleanupDb();
  process.exit();
});

test('Workspace E2E Puppeteer Tests', async (t) => {
  cleanupDb();
  await runInstallWorkspace();
  await runServe();

  const browser = await puppeteer.launch({
    headless: HEADLESS ? 'new' : false,
    args: ['--no-sandbox', '--disable-setuid-sandbox']
  });
  const page = await browser.newPage();

  // Pipe browser logs and errors to terminal for debugging
  page.on('console', msg => console.log('PAGE LOG:', msg.text()));
  page.on('pageerror', err => console.error('PAGE ERROR STACK:', err.stack || err.toString()));

  const waitForSelector = async (selector, timeout = 10000) => {
    await page.waitForSelector(selector, { timeout });
  };

  try {
    // -------------------------------------------------------------
    // 1. Interactive SQL Editor & Results Verification (US1)
    // -------------------------------------------------------------
    try {
      await t.test('US1: Run SQL Query & Verify Tabular Results and Error Handling', async () => {
        await page.goto(`http://127.0.0.1:${PORT}/workspace/sql_editor`);
        await waitForSelector('#sql-editor-textarea');

        console.log('US1: Executing CALL pizza_tx.simulate_random_order()...');
        await page.evaluate(() => {
          const textarea = document.getElementById('sql-editor-textarea');
          textarea.value = 'CALL pizza_tx.simulate_random_order();';
          textarea.dispatchEvent(new Event('input', { bubbles: true }));
          textarea.dispatchEvent(new Event('change', { bubbles: true }));
        });
        await page.click('button[type="submit"]');

        // Wait for query results container to update
        await page.waitForFunction(() => {
          const res = document.getElementById('query-results');
          return res && (res.querySelector('table') || res.querySelector('.alert-success') || res.querySelector('.alert-error'));
        }, { timeout: 10000 });

        // Give background flusher time to write the logs to the DB
        console.log('US1: Waiting for async flusher...');
        await new Promise(r => setTimeout(r, 2000));

        console.log('US1: Executing SELECT FROM system.logs...');
        // Run standard select query
        await page.evaluate(() => {
          const textarea = document.getElementById('sql-editor-textarea');
          textarea.value = 'SELECT * FROM system.logs;';
          textarea.dispatchEvent(new Event('input', { bubbles: true }));
          textarea.dispatchEvent(new Event('change', { bubbles: true }));
        });
        await page.click('button[type="submit"]');

        // Wait for results table
        await waitForSelector('#query-results table');
        const tableText = await page.evaluate(() => document.querySelector('#query-results table').innerText);
        console.log('US1: logs query tableText preview:', tableText.substring(0, 200));
        assert.match(tableText, /oxibase/i, 'Table should contain standard oxibase logs');
        assert.match(tableText, /Simulated/i, 'Table should contain simulated order logs');

        console.log('US1: Executing malformed query...');
        // Test malformed SQL syntax and error box rendering
        await page.evaluate(() => {
          const textarea = document.getElementById('sql-editor-textarea');
          textarea.value = 'SELECT malformed_syntax FROM non_existent_table_xyz;';
          textarea.dispatchEvent(new Event('input', { bubbles: true }));
          textarea.dispatchEvent(new Event('change', { bubbles: true }));
        });
        await page.click('button[type="submit"]');

        // Wait for error box
        await waitForSelector('#query-results .alert-error');
        const errorText = await page.evaluate(() => document.querySelector('#query-results .alert-error').innerText);
        console.log('US1: malformed query errorText:', errorText);
        assert.match(errorText, /Query Error/i, 'Error box should be rendered with Query Error header');
      });
    } catch (e) {
      console.error('US1 Test block failed to run:', e);
    }

    // -------------------------------------------------------------
    // 2. Logs Explorer & Trace Timeline Verification (US2)
    // -------------------------------------------------------------
    try {
      await t.test('US2: Verify Logs Severity Metrics & Gantt Trace Tree', async () => {
        // Navigate to logs explorer
        console.log('US2: Navigating to logs explorer...');
        await page.goto(`http://127.0.0.1:${PORT}/workspace/observe/logs`);
        
        console.log('US2: Waiting for body log...');
        await new Promise(r => setTimeout(r, 1000));
        const bodyHTML = await page.evaluate(() => document.body.innerHTML);
        console.log('US2: body HTML preview:', bodyHTML.substring(0, 500));

        console.log('US2: Waiting for log severity counters to populate...');
        // Wait for histogram elements to populate (via the async fetch) and verify counts
        await page.waitForFunction(() => {
          const errText = document.getElementById('hist-error')?.textContent;
          const warnText = document.getElementById('hist-warn')?.textContent;
          const infoText = document.getElementById('hist-info')?.textContent;
          const debugText = document.getElementById('hist-debug')?.textContent;
          if (!errText || !warnText || !infoText || !debugText) return false;
          return !errText.includes('ERROR: 0') && !warnText.includes('WARN: 0') && !infoText.includes('INFO: 0') && !debugText.includes('DEBUG: 0');
        }, { timeout: 10000 });

        const errText = await page.evaluate(() => document.getElementById('hist-error').textContent.trim());
        const warnText = await page.evaluate(() => document.getElementById('hist-warn').textContent.trim());
        const infoText = await page.evaluate(() => document.getElementById('hist-info').textContent.trim());
        const debugText = await page.evaluate(() => document.getElementById('hist-debug').textContent.trim());

        console.log(`US2: Histogram counts: ${errText}, ${warnText}, ${infoText}, ${debugText}`);
        assert.match(errText, /ERROR: [1-9]\d*/, 'ERROR severity count should be non-zero');
        assert.match(warnText, /WARN: [1-9]\d*/, 'WARN severity count should be non-zero');
        assert.match(infoText, /INFO: [1-9]\d*/, 'INFO severity count should be non-zero');
        assert.match(debugText, /DEBUG: [1-9]\d*/, 'DEBUG severity count should be non-zero');

        console.log('US2: Navigating to traces catalog to grab a dynamic trace ID...');
        // Navigate to traces catalog to grab a dynamic trace ID
        await page.goto(`http://127.0.0.1:${PORT}/workspace/observe/traces`);
        await waitForSelector('table tbody tr td.font-mono');

        const traceId = await page.evaluate(() => {
          const cell = document.querySelector('table tbody tr td.font-mono');
          return cell ? cell.textContent.trim() : null;
        });
        console.log('US2: Found traceId:', traceId);
        assert.ok(traceId, 'Should find at least one trace ID');

        // Navigate to trace details view
        await page.goto(`http://127.0.0.1:${PORT}/workspace/traces/${traceId}`);
        await waitForSelector('#tree-container .gantt-timing-bar');

        const ganttBarsCount = await page.evaluate(() => document.querySelectorAll('#tree-container .gantt-timing-bar').length);
        console.log('US2: ganttBarsCount:', ganttBarsCount);
        assert.equal(ganttBarsCount >= 1, true, 'Should render at least 1 Gantt timing bar for trace spans');
      });
    } catch (e) {
      console.error('US2 Test block failed to run:', e);
    }

    // -------------------------------------------------------------
    // 3. Interactive Debugger Handshake & Breakpoints (US3)
    // -------------------------------------------------------------
    await t.test('US3: Verify Debugger Handshake & Breakpoint Toggle', async () => {
      // Navigate to debugger panel for SYNC_DAILY_SUMMARY procedure
      await page.goto(`http://127.0.0.1:${PORT}/workspace/debugger?procedure_name=SYNC_DAILY_SUMMARY`);
      await waitForSelector('#cm-editor');

      // Assert code loaded in the editor
      const editorText = await page.evaluate(() => document.querySelector('#cm-editor .cm-content').innerText);
      console.log('US3: Editor text length:', editorText.length);
      assert.match(editorText, /daily_sales_summary/i, 'Editor should display the procedure code');

      // Simulate clicking on the first line gutter to toggle a breakpoint
      await waitForSelector('.cm-gutters .cm-gutterElement');
      
      console.log('US3: Toggling breakpoint on first line...');
      await page.evaluate(() => {
        const lineNumbers = Array.from(document.querySelectorAll('.cm-gutters .cm-gutterElement'));
        const firstLine = lineNumbers.find(el => el.innerText.trim() === '1');
        if (firstLine) {
          const rect = firstLine.getBoundingClientRect();
          const evt = new MouseEvent('mousedown', {
            clientX: rect.left + rect.width / 2,
            clientY: rect.top + rect.height / 2,
            bubbles: true
          });
          firstLine.dispatchEvent(evt);
        }
      });

      // Verify red highlight state gets added
      await page.waitForFunction(() => {
        const gutterElements = Array.from(document.querySelectorAll('.cm-gutters .cm-gutterElement'));
        const firstLine = gutterElements.find(el => el.innerText.trim() === '1');
        return firstLine && firstLine.style.color === 'red';
      }, { timeout: 10000 });

      const isBreakpointSet = await page.evaluate(() => {
        const gutterElements = Array.from(document.querySelectorAll('.cm-gutters .cm-gutterElement'));
        const firstLine = gutterElements.find(el => el.innerText.trim() === '1');
        return firstLine && firstLine.style.color === 'red';
      });
      assert.equal(isBreakpointSet, true, 'First line number gutter should be highlighted red indicating breakpoint set');
    });

    // -------------------------------------------------------------
    // 4. Database Catalog & Schema Explorer (US4)
    // -------------------------------------------------------------
    try {
      await t.test('US4: Sidebar Data Clicks & Routing to Grid', async () => {
        // Go to sidebar data
        console.log('US4: Navigating to sidebar data...');
        await page.goto(`http://127.0.0.1:${PORT}/workspace/sidebar/data`);
        
        console.log('US4: Waiting 1s to inspect HTML...');
        await new Promise(r => setTimeout(r, 1000));
        const sidebarHTML = await page.evaluate(() => document.getElementById('sidebar-content')?.innerHTML);
        console.log('US4: sidebar HTML:', sidebarHTML);

        await waitForSelector('#sidebar-content details');

        // Verify schemas are listed (like PIZZA_TX)
        const sidebarText = await page.evaluate(() => document.getElementById('sidebar-content').textContent);
        console.log('US4: sidebarText length:', sidebarText.length);
        assert.match(sidebarText, /pizza_tx/i, 'Sidebar data should list pizza_tx schema');
        assert.match(sidebarText, /customer/i, 'Sidebar data should list customer table');

        // Click on customer table link
        console.log('US4: Clicking customer table link...');
        await page.evaluate(() => {
          const links = Array.from(document.querySelectorAll('#sidebar-content a'));
          const customerLink = links.find(el => el.textContent.includes('customer'));
          if (customerLink) {
            customerLink.click();
          }
        });

        // Wait for data grid to load in main view
        await waitForSelector('main table');
        const gridText = await page.evaluate(() => document.querySelector('main table').textContent);
        console.log('US4: gridText preview:', gridText.substring(0, 100));
        assert.match(gridText, /Alice|Bob|Charlie/i, 'Data grid should display customer rows');
      });
    } catch (e) {
      console.error('US4 Test block failed to run:', e);
    }

  } finally {
    await browser.close();
    killServeProcess();
    cleanupDb();
  }
});
