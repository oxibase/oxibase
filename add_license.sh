#!/bin/bash
HEADER="// Copyright 2025 Stoolap Contributors
// Copyright 2025 Oxibase Contributors
//
// Licensed under the Apache License, Version 2.0 (the \"License\");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License.
// See the License for the specific language governing permissions and
// limitations under the License.
"

for file in "tests/server_test.rs" "src/server/handlers.rs" "src/server/mod.rs"; do
  echo "$HEADER" | cat - "$file" > temp && mv temp "$file"
done
