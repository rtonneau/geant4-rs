# Ticket 04: reaction-placement

**Model:** heavy
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] Placement preserves source product order and uses the Geant4 reaction-site branch for one product, parent sites for two, and reaction site then parent sites for three; zero products produce no positions.
- [ ] Conditional parent placement follows the literal Geant4 11.4.1 expression, including the approved Gaussian argument, with documented dimensional limitation and typed errors for unrepresentable/ambiguous cases.
- [ ] Tests cover zero separation, unequal and zero individual diffusion, equal-time and positive-delay branches, one/two/three products, and zero-product unary reactions without inventing Brownian placement.

**Files to Touch:**
- `crates/g4-chemistry/src/placement.rs` (new)
- `crates/g4-chemistry/src/reaction_network.rs`
- `crates/g4-chemistry/src/error.rs`
- `crates/g4-chemistry/src/lib.rs`
- `crates/g4-chemistry/tests/reaction_placement.rs` (new)

**Verification Step:**

Run:
```bash
cargo test -p g4-chemistry --test reaction_placement
```

Expected:
All source-branch and degenerate placement tests pass with exit 0.

**Notes:**

Audit `G4DNAIRT::MakeReaction` lines 393–495. The primitive receives validated reactants, positions, elapsed time and caller RNG; it does not advance a whole history. In the unary case the source creates a zero-diffusion fake bulk partner at the mobile parent's position.
