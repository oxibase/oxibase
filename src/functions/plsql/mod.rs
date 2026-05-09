// Copyright 2025 Oxibase Contributors
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

//! PL/SQL Native Interpreter
//!
//! This module implements a dedicated parser and interpreter for a PL/SQL-like
//! procedural language. It is designed to maintain execution state (call stack,
//! local variables) in a way that allows a future Debug Adapter Protocol (DAP)
//! server to attach and step through the code.

pub mod ast;
pub mod backend;
pub mod env;
pub mod interpreter;
pub mod parser;
