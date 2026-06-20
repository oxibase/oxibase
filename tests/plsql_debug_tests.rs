// Copyright 2025 Oxibase Contributors

use oxibase::functions::plsql::env::Environment;
use oxibase::functions::plsql::interpreter::{DebugAdapterHook, PlSqlInterpreter};
use oxibase::functions::plsql::parser::PlSqlParser;
use oxibase::functions::FunctionRegistry;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

struct MockDebugHook {
    lines_hit: Mutex<Vec<usize>>,
    action: Mutex<String>, // "pause", "step", or "continue"
    cvar: Condvar,
}

impl MockDebugHook {
    fn new() -> Self {
        Self {
            lines_hit: Mutex::new(Vec::new()),
            action: Mutex::new("continue".to_string()),
            cvar: Condvar::new(),
        }
    }

    fn set_action(&self, new_action: &str) {
        let mut action = self.action.lock().unwrap();
        *action = new_action.to_string();
        self.cvar.notify_one();
    }
}

impl DebugAdapterHook for MockDebugHook {
    fn on_statement_before_eval(&self, line_number: usize, env: &Environment) {
        self.lines_hit.lock().unwrap().push(line_number);
        let scopes = env.to_dap_scopes();
        println!("Hit line {}, scopes: {:?}", line_number, scopes);

        let mut action = self.action.lock().unwrap();
        while *action == "pause" {
            action = self.cvar.wait(action).unwrap();
        }

        // If stepping, pause on the next statement
        if *action == "step" {
            *action = "pause".to_string();
        }
    }
}

#[test]
fn test_plsql_debugger_hook() {
    let code = r#"
    DECLARE
        a INT := 5;
    BEGIN
        a := a + 1;
        a := a * 2;
    END;
    "#;

    let mut parser = PlSqlParser::new(code);
    let block = parser.parse().unwrap();

    let mut env = Environment::new();
    let registry = Arc::new(FunctionRegistry::new());

    let hook = Arc::new(MockDebugHook::new());

    // Start paused
    hook.set_action("pause");

    let interpreter = PlSqlInterpreter::new(registry, None).with_debug_hook(hook.clone());

    let hook_clone = hook.clone();
    let handle = thread::spawn(move || {
        interpreter.execute(&block, &mut env).unwrap();
    });

    // Allow thread to reach the first statement
    thread::sleep(std::time::Duration::from_millis(50));

    // Step over the first statement
    hook_clone.set_action("step");
    thread::sleep(std::time::Duration::from_millis(50));

    // Continue until the end
    hook_clone.set_action("continue");

    handle.join().unwrap();

    let hits = hook.lines_hit.lock().unwrap().clone();
    assert!(!hits.is_empty(), "No breakpoints hit");
    println!("Lines hit: {:?}", hits);
}

#[test]
fn test_plsql_database_debugger() {
    use oxibase::common::debug::{DebugController, ResumeAction};
    use oxibase::Database;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    let db = Database::open("memory://plsql_debug_db").unwrap();

    db.execute(
        r#"
        CREATE PROCEDURE debug_plsql_proc()
        LANGUAGE plsql
        AS '
        DECLARE
            x INT := 10;
            y INT := 20;
        BEGIN
            x := x + 10;
            y := y + 20;
        END;
        ';
        "#,
        (),
    )
    .unwrap();

    let dc = Arc::new(DebugController::new());
    // Set breakpoint on line 6 (corresponds to "x := x + 10;")
    dc.set_breakpoints("DEBUG_PLSQL_PROC", vec![6]);

    let db_clone = db.clone();
    let dc_clone = Arc::clone(&dc);

    let handle = thread::spawn(move || {
        oxibase::functions::context::with_http_headers_and_debug(
            HashMap::new(),
            Some(dc_clone),
            || {
                db_clone.execute("CALL debug_plsql_proc();", ()).unwrap();
            },
        );
    });

    // Let execution reach the breakpoint (poll with timeout)
    for _ in 0..100 {
        {
            let state = dc.pause_mutex.lock().unwrap();
            if state.is_paused {
                break;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }

    // Verify that the thread is paused and we can see local variables
    {
        let state = dc.pause_mutex.lock().unwrap();
        assert!(
            state.is_paused,
            "PL/SQL execution should be paused at the breakpoint"
        );

        if let Some(ref locals) = state.current_locals {
            assert_eq!(locals["x"], "10");
        } else {
            panic!("No local variables captured at PL/SQL breakpoint");
        }
    }

    // Issue a StepOver command
    dc.resume(ResumeAction::StepOver);
    thread::sleep(Duration::from_millis(50));

    // Give it a moment to step and pause on the next line (line 7) (poll with timeout)
    for _ in 0..100 {
        {
            let state = dc.pause_mutex.lock().unwrap();
            if state.is_paused {
                break;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }

    // Verify that the thread is paused at line 7 (even though there is no breakpoint there)
    {
        let state = dc.pause_mutex.lock().unwrap();
        assert!(
            state.is_paused,
            "PL/SQL execution should be paused after StepOver"
        );

        if let Some(ref locals) = state.current_locals {
            assert_eq!(locals["x"], "20");
            assert_eq!(locals["y"], "20");
        } else {
            panic!("No local variables captured after PL/SQL StepOver");
        }
    }

    // Resume execution with Continue
    dc.resume(ResumeAction::Continue);

    // Join thread
    handle.join().unwrap();
}
