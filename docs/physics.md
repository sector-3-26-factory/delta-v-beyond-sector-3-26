# Physics notes

The flight model of *Delta-V beyond Sector 3.26* is intentionally
Newtonian. This document captures the rules so that gameplay, AI, netcode
and UI all share one mental model.

## Core rules

1. **Newton's second law applies.** A ship is a rigid body with mass `m`
   and moment of inertia tensor `I`. Linear motion follows `F = m * a`,
   rotational motion follows `tau = I * alpha`.
2. **No arbitrary top speed.** Velocity accumulates as long as thrust is
   applied. The only practical limits are fuel/heat budgets (future) and
   the size of the playable volume.
3. **Momentum is preserved.** Cutting the throttle does *not* slow the
   ship down. Decelerating requires reverse thrust.
4. **All thrust is local.** Inputs map to forces in the ship's local
   frame (forward/back, strafe, vertical, pitch/yaw/roll torque). The
   physics engine integrates them in world space.

## Inputs and effectors

| Input axis        | Effect                                        |
| ----------------- | --------------------------------------------- |
| Throttle forward  | +Z local force                                |
| Throttle reverse  | -Z local force                                |
| Strafe X / Y      | +/- X, +/- Y local force                      |
| Pitch / Yaw / Roll| Torque around local X / Y / Z                 |

Each effector has a maximum force/torque; input is clamped to `[-1, 1]`
and scaled.

## Flight assist (optional, accessibility)

Pure Newtonian flight is unforgiving. We offer an opt-in "flight assist"
mode:

- **Linear damping**: when no translational input is given on an axis,
  apply a counter-force proportional to the velocity component on that
  axis, up to the available thrust on the opposite effector.
- **Rotational damping**: analogous for angular velocity.

Flight assist never violates conservation laws by magic; it only commands
the existing thrusters to null out residual motion.

## Projectiles

- A projectile inherits the firing ship's world velocity at the moment of
  firing: `v_projectile = v_ship + R_ship * v_muzzle_local`.
- Projectiles are rigid bodies and obey the same physics rules.
- No drag in vacuum.
- Projectiles can apply damage to other rigid bodies on collision.
- Damage is applied via the `Health` component; entities are destroyed when
  health reaches zero.

## Determinism

- Use a fixed timestep for the physics schedule.
- Avoid relying on wall-clock time inside gameplay systems.
- Keep floating-point operations on a single precision policy (f32 in
  Bevy by default) and avoid platform-specific intrinsics.

## Units

- Length: meters (m)
- Time: seconds (s)
- Mass: kilograms (kg)
- Force: newtons (N)
- Angles: radians internally; UI may convert to degrees.

## Open questions

- Should we model fuel / propellant as a finite resource from the start,
  or add it later as a balance knob?
- Heat / thermal management as a gameplay layer?
- Relativistic effects: explicitly **out of scope** (non-relativistic
  Newtonian only).
