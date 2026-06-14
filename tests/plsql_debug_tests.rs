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
