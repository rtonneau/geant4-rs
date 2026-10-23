# Ticket 02: molecule-and-diffusion

**Model:** medium
**Model (Jev):** sonnet-5.5 (confidence 0.27)
**Effort:** high

**Acceptance Criteria:**
- [ ] A chemistry-layer molecule wrapper combines an unchanged `SpeciesInstance` with a stable `MoleculeId`; pair construction rejects equal IDs and permits distinct A+A molecules.
- [ ] Free 3D diffusion samples each axis from `Normal(0, sqrt(2 D dt))` using the existing caller-provided RNG helpers.
- [ ] `dt = 0` or `D = 0` returns the input position bit-for-bit and consumes no random draws; negative/non-finite inputs and non-finite output are typed errors.
- [ ] Fixed-seed tests cover draw count, variance/isotropy, translation, validation, and A+A identity behavior.

**Files to Touch:**
- `crates/dna-chemistry/src/molecule.rs`
- `crates/dna-chemistry/src/diffusion.rs`
- `crates/dna-chemistry/src/error.rs`
- `crates/dna-chemistry/src/lib.rs`
- `crates/dna-chemistry/tests/diffusion.rs`

**Verification Step:**

Run:
```bash
cargo test -p dna-chemistry --test diffusion
```

Expected:
All identity, zero-draw, validation, Gaussian moment, and isotropy tests pass.

**Notes:**

Do not change `dna_core::SpeciesInstance` or M6 serialization. The diffusion kernel accepts `&mut impl RngCore`; stream ownership stays with the caller. Use TDD and commit as `feat: add molecule identity and free diffusion`.
