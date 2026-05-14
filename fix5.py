import re

with open("src/executor/context.rs", "r") as f:
    content = f.read()

content = content.replace("    view_depth: usize,\n    /// Query execution depth (0 = top-level query, >0 = subquery/nested)", "    view_depth: usize,\n    /// Whether this context is executing internal system queries\n    is_internal: bool,\n    /// Query execution depth (0 = top-level query, >0 = subquery/nested)")
content = content.replace("            view_depth: 0,\n            query_depth: 0,", "            view_depth: 0,\n            is_internal: false,\n            query_depth: 0,")
content = content.replace("            timeout_ms: self.timeout_ms,\n            view_depth: self.view_depth,", "            timeout_ms: self.timeout_ms,\n            is_internal: self.is_internal,\n            view_depth: self.view_depth,")
content = content.replace("            timeout_ms: self.timeout_ms,\n            view_depth: self.view_depth + 1,", "            timeout_ms: self.timeout_ms,\n            is_internal: self.is_internal,\n            view_depth: self.view_depth + 1,")

METHODS = """
    /// Returns the internal execution flag
    pub fn is_internal(&self) -> bool {
        self.is_internal
    }
"""

content = content.replace("    pub fn view_depth(&self) -> usize {\n        self.view_depth\n    }", "    pub fn view_depth(&self) -> usize {\n        self.view_depth\n    }\n" + METHODS)

METHODS2 = """
    /// Set internal execution flag
    pub fn with_internal(mut self, is_internal: bool) -> Self {
        self.ctx.is_internal = is_internal;
        self
    }
"""

content = content.replace("    /// Build the execution context", METHODS2 + "\n    /// Build the execution context")

with open("src/executor/context.rs", "w") as f:
    f.write(content)
