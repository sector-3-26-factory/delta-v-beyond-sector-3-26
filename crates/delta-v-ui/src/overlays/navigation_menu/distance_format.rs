// AGENTS: before modifying this file, read AGENTS.md at the repository root.
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

//! Distance formatting utilities for navigation menu display.

/// Formats a distance in meters to a human-readable string with appropriate SI units.
///
/// Units used: m, km, Mm, Gm, Tm, Pm (SI prefixes), ly (lightyear), pc (parsec).
/// The number part is kept to 3-4 digits for readability.
///
/// # Arguments
/// * `distance` - Distance in meters (must be non-negative)
///
/// # Returns
/// Formatted string like "123 m", "45.6 km", "7.89 Mm", "1.2 ly", "3.4 pc"
#[must_use]
pub fn format_distance(distance: f32) -> String {
    // Units: m, km, Mm, Gm, Tm, Pm (SI prefixes), ly (lightyear), pc (parsec).
    // 1 ly ≈ 9.461e15 m, 1 pc ≈ 3.086e16 m.
    if distance < 1_000.0 {
        format!("{distance:.0} m")
    } else if distance < 1_000_000.0 {
        format!("{:.1} km", distance / 1_000.0)
    } else if distance < 1_000_000_000.0 {
        format!("{:.1} Mm", distance / 1_000_000.0)
    } else if distance < 1_000_000_000_000.0 {
        format!("{:.1} Gm", distance / 1_000_000_000.0)
    } else if distance < 1_000_000_000_000_000.0 {
        format!("{:.1} Tm", distance / 1_000_000_000_000.0)
    } else if distance < 10_000_000_000_000_000.0 {
        format!("{:.1} Pm", distance / 1_000_000_000_000_000.0)
    } else if distance < 100_000_000_000_000_000.0 {
        // Lightyears: 1 ly ≈ 9.461e15 m
        format!("{:.1} ly", distance / 9_461_000_000_000_000.0)
    } else {
        // Parsecs: 1 pc ≈ 3.086e16 m
        format!("{:.1} pc", distance / 30_860_000_000_000_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_distance_meters() {
        assert_eq!(format_distance(0.0), "0 m");
        assert_eq!(format_distance(1.0), "1 m");
        assert_eq!(format_distance(999.0), "999 m");
    }

    #[test]
    fn test_format_distance_kilometers() {
        assert_eq!(format_distance(1_000.0), "1.0 km");
        assert_eq!(format_distance(1_500.0), "1.5 km");
        assert_eq!(format_distance(999_999.0), "1000.0 km");
    }

    #[test]
    fn test_format_distance_megameters() {
        assert_eq!(format_distance(1_000_000.0), "1.0 Mm");
        assert_eq!(format_distance(5_000_000.0), "5.0 Mm");
    }

    #[test]
    fn test_format_distance_gigameters() {
        assert_eq!(format_distance(1_000_000_000.0), "1.0 Gm");
    }

    #[test]
    fn test_format_distance_terameters() {
        assert_eq!(format_distance(1_000_000_000_000.0), "1.0 Tm");
    }

    #[test]
    fn test_format_distance_petameters() {
        assert_eq!(format_distance(1_000_000_000_000_000.0), "1.0 Pm");
        assert_eq!(format_distance(9_000_000_000_000_000.0), "9.0 Pm");
    }

    #[test]
    fn test_format_distance_lightyears() {
        // 1 ly ≈ 9.461e15 m
        // Boundary: Pm is < 10_000_000_000_000_000, ly is >= 10_000_000_000_000_000 and < 100_000_000_000_000_000
        assert_eq!(format_distance(10_000_000_000_000_000.0), "1.1 ly");
        assert_eq!(format_distance(94_610_000_000_000_000.0), "10.0 ly");
    }

    #[test]
    fn test_format_distance_parsecs() {
        // 1 pc ≈ 3.086e16 m
        // Boundary: ly is < 100_000_000_000_000_000, pc is >= 100_000_000_000_000_000
        assert_eq!(format_distance(100_000_000_000_000_000.0), "3.2 pc");
        assert_eq!(format_distance(308_600_000_000_000_000.0), "10.0 pc");
    }
}
