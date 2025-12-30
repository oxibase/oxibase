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

//! Scripting backend tests
//!
//! Tests for optional scripting backends (Deno, Python) that are only run
//! when the corresponding features are enabled.

#[cfg(feature = "deno")]
pub mod deno_tests;
#[cfg(feature = "python")]
pub mod python_tests;