// See AGENTS.md
//
// Delta-V beyond Sector 3.26
// Copyright (C) 2025  Cute-Donkey
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Engine constants for physics simulation.
//!
//! These are immutable, compile-time values that define the behaviour of the
//! physics engine itself. They are **not** gameplay knobs (which belong in JSON
//! schemas per ADR-0014).
//!
//! See ADR-0014 (Engine constants vs. gameplay values) and
//! ADR-0017 (Fixed timestep and determinism).

/// Fixed simulation timestep in Hz.
///
/// All physics and simulation systems run at this frequency, independent of
/// the rendering frame rate. Higher values give more precision but cost more
/// CPU per frame.
///
/// Per ADR-0017, this is chosen to be high enough for fast projectiles
/// (e.g. > 1 km/s) to remain stable, and low enough to be affordable on
/// modest hardware.
pub const FIXED_TIMESTEP_HZ: u32 = 60;

/// Maximum number of catch-up ticks per render frame.
///
/// If the renderer falls behind (e.g., a frame takes 100 ms), the `FixedUpdate`
/// schedule will run multiple times to catch up. This bound prevents a
/// "spiral of death" where a brief hiccup causes runaway catch-up ticks,
/// which stalls the renderer further.
///
/// Per ADR-0017, this is a tunable parameter. The initial value of 4 ticks
/// means a frame can fall at most 4/60 ≈ 67 ms behind before the catch-up
/// is capped. This is a reasonable trade-off: normal stutters are absorbed,
/// but runaway is prevented.
pub const CATCH_UP_TICKS_MAX: u32 = 4;

/// Gravitational constant G in SI units (m³/(kg⋅s²)).
///
/// This is the standard gravitational constant from physics. It is used to
/// compute gravitational acceleration between massive bodies (per ADR-0009).
pub const GRAVITATIONAL_CONSTANT: f32 = 6.674e-11;

/// Radius beyond which gravity contributions are culled to zero.
///
/// Entities separated by more than this distance do not exert gravitational
/// force on each other. This reduces the O(N²) gravity-interaction cost.
///
/// Initial value is 1000 km (`1_000_000` m). This is large enough to encompass
/// most gameplay scenarios (planets, stations, nearby asteroids) while
/// excluding truly distant objects.
pub const GRAVITY_CUTOFF_RADIUS_M: f32 = 1_000_000.0;
