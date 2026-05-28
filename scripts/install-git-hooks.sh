#!/usr/bin/env bash
# See AGENTS.md
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
# install-git-hooks.sh
#
# Configures the local Git clone to use the project's hooks in
# .githooks/. Run this once after cloning:
#
#   ./scripts/install-git-hooks.sh
#
# The hooks enforce cargo fmt, cargo clippy, cargo test and cargo deny
# before every commit (ADR-0034).

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HOOKS_DIR="$REPO_ROOT/.githooks"

if [ ! -d "$HOOKS_DIR" ]; then
    echo "ERROR: hooks directory not found: $HOOKS_DIR" >&2
    exit 1
fi

git -C "$REPO_ROOT" config --local core.hooksPath .githooks

echo "Git hooks installed. core.hooksPath -> .githooks"
echo "The following hooks are active:"
for hook in "$HOOKS_DIR"/*; do
    echo "  $(basename "$hook")"
done
