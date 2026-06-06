---
description: "ai driven review - prepare for human review"
---

Carefully re-examine the code against each ADR rule in @ARCHITECTURAL_RULES.md. Every role counts, so read @ARCHITECTURAL_RULES.md carefully. Use @git-changes to read changed code.

Run `bash scripts/check-forbidden-patterns.sh` to verify ADR-0013 compliance (no silent fallbacks via Default trait).