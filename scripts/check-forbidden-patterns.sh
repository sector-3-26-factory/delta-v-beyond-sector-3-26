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
# Currently enforces ADR-0013 (no silent fallbacks) and ADR-0056 (benchmark
# declarations, naming and tooling).
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

# ---------------------------------------------------------------------------
# ADR-0056: Benchmark declarations.
#
# A benchmark answers one of three questions, and what distinguishes them is
# what the number is compared against. Because the three need different
# baselines and different tooling, a benchmark file must say which one it is
# for, and the directory it sits in must agree. The layout is
#
#     crates/<crate>/src/<purpose>/<subject>_benchmarks_tests.rs
#
# so the directory carries the purpose and the file name carries the subject,
# which lets a single purpose hold as many benchmark files as it needs. This
# block checks the file layout, the declaration, the closed vocabulary, the
# purpose-to-baseline mapping, the `bench` feature gate and the `criterion`
# ban. Note that no `benches/` target is used: benchmarks are ordinary modules
# gated behind the `bench` cargo feature.
# ---------------------------------------------------------------------------

# The baseline that a purpose is required to compare against.
required_baseline() {
    case "$1" in
        hardware-reference)  echo "named-machine" ;;
        regression-tracking) echo "own-history" ;;
        approach-comparison) echo "sibling-variants-same-run" ;;
        *)                   echo "" ;;
    esac
}

# The purpose-directory name for a purpose (snake_case, per ADR-0023).
purpose_dir_name() {
    case "$1" in
        hardware-reference)  echo "hardware_reference" ;;
        regression-tracking) echo "regression_tracking" ;;
        approach-comparison) echo "approach_comparison" ;;
        *)                   echo "" ;;
    esac
}

# The purpose a directory name stands for, or empty if it is not a purpose.
purpose_of_dir() {
    case "$1" in
        hardware_reference)  echo "hardware-reference" ;;
        regression_tracking) echo "regression-tracking" ;;
        approach_comparison) echo "approach-comparison" ;;
        *)                   echo "" ;;
    esac
}

# Walk up from a file until the directory holding its Cargo.toml is reached.
crate_root_of() {
    local dir
    dir="$(dirname "$1")"
    while [ "$dir" != "/" ] && [ ! -f "$dir/Cargo.toml" ]; do
        dir="$(dirname "$dir")"
    done
    echo "$dir"
}

check_benchmarks() {
    local found=0
    local file base dir_name dir_purpose purpose baseline missing crate_root

    while IFS= read -r -d '' file; do
        base="$(basename "$file")"
        dir_name="$(basename "$(dirname "$file")")"
        dir_purpose="$(purpose_of_dir "$dir_name")"

        # 1. The file name must name a subject and end in _benchmarks_tests.rs.
        case "$base" in
            *_benchmarks_tests.rs) ;;
            *)
                echo ""
                echo "FORBIDDEN in $file: benchmark file name (violates ADR-0056)."
                echo "Fix: a benchmark file is named after its subject and ends in"
                echo "     _benchmarks_tests.rs, e.g. collision_benchmarks_tests.rs."
                echo "     The purpose belongs in the directory, not in the file name."
                found=1
                continue
                ;;
        esac

        # 1a. The file name must not repeat the purpose directory name.
        case "$base" in
            "$dir_name"_*)
                echo ""
                echo "FORBIDDEN in $file: file name repeats the purpose directory '$dir_name' (violates ADR-0056)."
                echo "Fix: name the file after its subject, e.g. collision_benchmarks_tests.rs."
                echo "     The purpose is already carried by the directory."
                found=1
                continue
                ;;
        esac

        # 2. The file must sit in a purpose directory.
        if [ -z "$dir_purpose" ]; then
            echo ""
            echo "FORBIDDEN in $file: not inside a purpose directory (violates ADR-0056)."
            echo "Fix: move it into the crate's approach_comparison/, regression_tracking/"
            echo "     or hardware_reference/ directory. One purpose per directory."
            found=1
            continue
        fi

        # 3. All four declaration keys must be present.
        missing=""
        for key in BENCHMARK-PURPOSE BENCHMARK-BASELINE BENCHMARK-SCENARIO BENCHMARK-RUN; do
            grep -q "^//! $key:" "$file" || missing="$missing $key"
        done
        if [ -n "$missing" ]; then
            echo ""
            echo "FORBIDDEN in $file: missing declaration key(s):$missing (violates ADR-0056)."
            echo "Fix: add '//! BENCHMARK-PURPOSE: ...', '//! BENCHMARK-BASELINE: ...',"
            echo "     '//! BENCHMARK-SCENARIO: ...' and '//! BENCHMARK-RUN: ...'"
            echo "     to the module documentation."
            found=1
            continue
        fi

        # 4. The declared purpose must be in the closed vocabulary.
        purpose="$(sed -n 's|^//! BENCHMARK-PURPOSE: *||p' "$file" | head -1)"
        if [ -z "$(required_baseline "$purpose")" ]; then
            echo ""
            echo "FORBIDDEN in $file: unknown BENCHMARK-PURPOSE '$purpose' (violates ADR-0056)."
            echo "Fix: use hardware-reference, regression-tracking or approach-comparison."
            found=1
            continue
        fi

        # 5. The baseline must be the one fixed by the purpose.
        baseline="$(sed -n 's|^//! BENCHMARK-BASELINE: *||p' "$file" | head -1)"
        if [ "$baseline" != "$(required_baseline "$purpose")" ]; then
            echo ""
            echo "FORBIDDEN in $file: BENCHMARK-BASELINE is '$baseline' but purpose"
            echo "     '$purpose' requires '$(required_baseline "$purpose")' (violates ADR-0056)."
            found=1
        fi

        # 6. The declared purpose must match the directory it sits in.
        if [ "$purpose" != "$dir_purpose" ]; then
            echo ""
            echo "FORBIDDEN in $file: BENCHMARK-PURPOSE is '$purpose' but the directory"
            echo "     '$dir_name' means '$dir_purpose' (violates ADR-0056)."
            echo "Fix: either declare '$dir_purpose', or move the file into"
            echo "     '$(purpose_dir_name "$purpose")/'."
            found=1
        fi

        # 7. criterion is banned in an approach-comparison benchmark.
        if [ "$purpose" = "approach-comparison" ]; then
            if grep -nE 'use criterion|criterion::|criterion_(main|group)!' "$file"; then
                echo ""
                echo "FORBIDDEN in $file: criterion used in an approach-comparison benchmark (violates ADR-0056)."
                echo "Fix: criterion compares a benchmark against its own previous run, which is"
                echo "     a regression-tracking baseline. approach-comparison benchmarks must"
                echo "     measure their variants interleaved in one process instead."
                found=1
            fi
        fi

        # 8. The purpose directory must be gated behind the `bench` feature.
        crate_root="$(crate_root_of "$file")"
        if ! grep -rn -B3 --include='*.rs' -F "mod $dir_name;" "$crate_root" 2>/dev/null \
             | grep -q 'feature = "bench"'; then
            echo ""
            echo "FORBIDDEN in $file: the '$dir_name' module is not gated behind the 'bench'"
            echo "     cargo feature (violates ADR-0056)."
            echo "Fix: declare it as #[cfg(all(test, feature = \"bench\"))] in the crate"
            echo "     root, so day-to-day cargo test never compiles or runs it."
            found=1
        fi
    done < <(find "$SRC_DIR" -path '*/src/*' -name '*benchmark*.rs' -print0)

    return $found
}

check_benchmarks || ERRORS=$((ERRORS + 1))

if [ "$ERRORS" -gt 0 ]; then
    echo ""
    echo "$ERRORS forbidden pattern(s) found. Aborting."
    exit 1
fi

echo "[check-forbidden-patterns] OK"