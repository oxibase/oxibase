#!/bin/bash
set -e

SPEC_FILE="/Users/gabriel.maeztu/repos/oxibase/specs/045-rhai-timestamp/spec.md"

# Check if ## Clarifications exists, if not add it after the first section (User Scenarios)
if ! grep -q "^## Clarifications" "$SPEC_FILE"; then
  # Insert before "## User Scenarios & Testing"
  sed -i '' '/^## User Scenarios & Testing/i\
## Clarifications\
\
### Session 2026-06-18\
\
' "$SPEC_FILE"
fi

# Add the bullet
sed -i '' '/^### Session 2026-06-18/a\
- Q: How should database timestamps (`Value::Timestamp`) be passed to and returned from Rhai scripts? → A: Map as a custom Rhai DateTime object with helper methods (Option C).\
' "$SPEC_FILE"

# Update Functional Requirements
sed -i '' '/- \*\*FR-003\*\*: System MUST expose Rhai.s `sleep` function for script execution delays./a\
- **FR-004**: System MUST map Oxibase `Value::Timestamp(DateTime<Utc>)` to a custom Rhai DateTime object during argument passing.\
- **FR-005**: System MUST support returning custom Rhai DateTime objects and mapping them back to Oxibase `Value::Timestamp`.\
' "$SPEC_FILE"

echo "Updated $SPEC_FILE"
