# Known Issues

This document tracks known issues that need to be addressed.

## Build Errors (91 compilation errors)

### Status: 🔴 **NEEDS FIXING**

### Summary

The project currently has 91 compilation errors, primarily borrow checker issues in the following modules:

1. **src/pso.rs** - Particle Swarm Optimization
2. **src/aco.rs** - Ant Colony Optimization
3. **src/gwo.rs** - Grey Wolf Optimizer

### Primary Issues

#### 1. Borrow Checker Errors (E0502)

**Location**: `src/pso.rs:251-253`

```rust
// ERROR: Cannot borrow `*self` as immutable because it is also borrowed as mutable
for particle in &mut self.particles {
    self.update_velocity(particle)?;  // ❌
    self.update_position(particle)?;  // ❌
}
```

**Solution**: Refactor to separate mutable and immutable borrows:

```rust
// Option 1: Use indices instead of iterators
for i in 0..self.particles.len() {
    let particle = &mut self.particles[i];
    self.update_velocity_index(i)?;
    self.update_position_index(i)?;
}

// Option 2: Extract required data before the loop
let velocities = self.calculate_all_velocities()?;
for (particle, velocity) in self.particles.iter_mut().zip(velocities) {
    particle.velocity = velocity;
    particle.position += velocity;
}
```

#### 2. Closure Borrowing Issues

**Location**: `src/pso.rs:598-599`

```rust
// ERROR: Cannot borrow `self.pso` as mutable while it's borrowed immutably in closure
let cost_fn = |wp: &[f32]| self.path_cost(wp);  // ❌ immutable borrow
let (best_waypoints, _cost) = self.pso.optimize(iterations, cost_fn)?;  // ❌ mutable borrow
```

**Solution**: Use interior mutability or restructure:

```rust
// Option 1: Split into separate methods
let costs: Vec<_> = waypoint_candidates
    .iter()
    .map(|wp| self.path_cost(wp))
    .collect();

let best_idx = costs.iter()
    .enumerate()
    .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
    .map(|(idx, _)| idx)
    .unwrap();

// Option 2: Use RefCell for interior mutability
use std::cell::RefCell;
struct PathPlanner {
    pso: RefCell<PSO>,
    // ...
}
```

#### 3. ACO Pheromone Deposition

**Location**: `src/aco.rs:497-499`

```rust
// ERROR: Cannot borrow as mutable while immutably borrowed
for ant in &self.ants {  // ❌ immutable borrow
    if ant.path.is_valid {
        self.deposit_pheromone(&ant.path)?;  // ❌ mutable borrow
    }
}
```

**Solution**: Collect paths first, then deposit:

```rust
let valid_paths: Vec<_> = self.ants
    .iter()
    .filter(|ant| ant.path.is_valid)
    .map(|ant| ant.path.clone())
    .collect();

for path in valid_paths {
    self.deposit_pheromone(&path)?;
}
```

### Dependency Issues (FIXED)

- ✅ Fixed `ed25519-dalek` `u32_backend` feature removal
- ✅ Fixed `x25519-dalek` `u32_backend` feature removal
- ✅ Added missing `smoltcp` medium features

### Action Items

- [ ] Fix borrow checker errors in PSO module
- [ ] Fix borrow checker errors in ACO module
- [ ] Fix borrow checker errors in GWO module
- [ ] Run `cargo clippy` and address all warnings
- [ ] Run `cargo fmt` to ensure consistent formatting
- [ ] Add comprehensive tests for fixed modules
- [ ] Update CI/CD to catch these issues

### Temporary Workaround

Until these are fixed, you can:

1. **Comment out problematic modules** in `src/lib.rs`:
   ```rust
   // pub mod pso;
   // pub mod aco;
   // pub mod gwo;
   ```

2. **Use the other modules** that are working:
   - Cryptography (crypto)
   - Networking (network)
   - Consensus (consensus)
   - Federated Learning (federated)
   - Basic swarm control (swarm)

### Timeline

**Priority**: High
**Est. Fix Time**: 2-4 hours
**Assigned**: TBD

### Related Issues

- [ ] #TODO: Create GitHub issue for PSO borrow checker errors
- [ ] #TODO: Create GitHub issue for ACO borrow checker errors
- [ ] #TODO: Create GitHub issue for GWO borrow checker errors

---

**Last Updated**: 2025-11-28
**Status**: Documentation and infrastructure complete, core algorithms need refactoring
