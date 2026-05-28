#!/usr/bin/env bash
# See AGENTS.md – enforces ADR constraints that cannot be expressed as compiler errors.
#
# Delta-V beyond Sector 3.26
# Copyright (C) 2025  Cute-Donkey
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.
#
# check-forbidden-patterns.sh
#
# Scans Rust sources for patterns that are banned by ADR constraints.
# Called from .githooks/pre-commit and CI (adr-compliance.yml).
# Run manually:
#
#   bash scripts/check-forbidden-patterns.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC_DIR="$REPO_ROOT/crates"

ERRORS=0

# ADR-0013: No silent fallbacks.
# Implementing the Default trait for config types silently papers over missing
# or invalid configuration instead of hard-crashing with a clear error.
# The argument "Rust defaults equal JSON defaults" is explicitly rejected:
# if a JSON default fails to apply the process MUST panic – never fall back.
#
# Both manual `impl Default` and `#[derive(Default)]` are forbidden.

# Files containing the marker "allow-default: <reason>" are explicitly
# exempted from the Default ban. The marker must appear in the file and
# its presence is itself auditable in code review.
# Example use-case: Bevy States require Default for state-machine init.

check_pattern() {
    local pattern="$1"
    local label="$2"
    local found=0
    while IFS= read -r -d '' file; do
        # Skip files that carry an explicit allow-default exemption marker.
        if grep -q "allow-default:" "$file"; then
            continue
        fi
        if grep -n "$pattern" "$file"; then
            echo ""
            echo "FORBIDDEN in $file: $label (violates ADR-0013 / no-silent-fallbacks)."
            echo "Fix: remove the Default impl and hard-error on missing config instead."
            echo "If Default is genuinely required (e.g. Bevy trait bound), add a line:"
            echo "  // allow-default: <reason>"
            echo "anywhere in the file."
            found=1
        fi
    done < <(find "$SRC_DIR" -name "*.rs" -print0)
    return $found
}

check_pattern "fn default()"        "'fn default()'"       || ERRORS=$((ERRORS + 1))
check_pattern "derive([^)]*Default" "'derive(Default)'"   || ERRORS=$((ERRORS + 1))

if [ "$ERRORS" -gt 0 ]; then
    echo ""
    echo "$ERRORS forbidden pattern(s) found. Aborting."
    exit 1
fi

echo "[check-forbidden-patterns] OK"