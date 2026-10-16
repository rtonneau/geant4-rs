// g4rs_ref: reference-data generator for the geant4-rs M2 validation (docs/07 V2, V3, V5).
//
// STATUS: UNTESTED. This file was written without access to a Geant4 installation and has never
// been compiled. It is derived from the examples extended/medical/dna/{mfp,range,dnaphysics}
// (Geant4 11.4.1). Expect to fix small API/compile problems; do NOT change the physics settings
// listed below, they define what the Rust implementation is compared against.
//
// Physics: G4EmDNAPhysics_option2 (+ G4DecayPhysics for particle construction), liquid water
// G4_WATER (1 g/cm3), electrons only. Atomic de-excitation (fluorescence/Auger) is switched
// OFF because geant4-rs M2 does not simulate it (docs/09 Q12); with it on, K-shell ionisations
// would book less than 539 eV locally and the deposited-energy comparison would differ for a
// documented reason.
//
// Modes (CSV written to <out.csv>; `#` lines are provenance comments):
//   xs    <out.csv>                       cross section per volume of each DNA process (V2, V3)
//   xslow <out.csv>                       M3: the same for all five cross-section processes (adds
//                                         vibexc = Sanche, attachment = Melton) between 1.5 eV and
//                                         150 eV, including the window boundaries (V2 for M3)
//   mfp   <out.csv> <events> <seed>       first interaction only, like examples/.../mfp (V5)
//   track <out.csv> <events> <seed> <cut_eV>
//                                         primary electron followed until its kinetic energy falls
//                                         below <cut_eV>; secondaries are killed (their energy is
//                                         summed); like examples/.../range with a tracking cut (V5)
//   slow  <out_prefix> <seed> [E_eV:events,E_eV:events,...]
//                                         M3: full slowing-down. The primary AND every secondary
//                                         are followed until they die (solvation or attachment);
//                                         nothing is killed or cut. Writes <out_prefix>_sums.csv
//                                         (per energy: sums and sums of squares per event),
//                                         <out_prefix>_spectra.csv (secondary electron energy
//                                         histograms by generation) and <out_prefix>_radial.csv
//                                         (energy deposited per radial shell). Same physics
//                                         settings as the other modes (option2, fluo/Auger off).

#include <algorithm>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <map>
#include <string>
#include <vector>

#include "G4DecayPhysics.hh"
#include "G4EmCalculator.hh"
#include "G4EmDNAPhysics_option2.hh"
#include "G4EmDNAChemistry_option3.hh"
#include "G4EmParameters.hh"
#include "G4Electron.hh"
#include "G4Event.hh"
#include "G4GeometryManager.hh"
#include "G4LogicalVolume.hh"
#include "G4LossTableManager.hh"
#include "G4NistManager.hh"
#include "G4PVPlacement.hh"
#include "G4ParticleGun.hh"
#include "G4PhysicalConstants.hh"
#include "G4ProductionCutsTable.hh"
#include "G4RunManagerFactory.hh"
#include "G4Sphere.hh"
#include "G4Step.hh"
#include "G4StepPoint.hh"
#include "G4StackManager.hh"
#include "G4SystemOfUnits.hh"
#include "G4Track.hh"
#include "G4VProcess.hh"
#include "G4Version.hh"
#include "G4UserSteppingAction.hh"
#include "G4UserEventAction.hh"
#include "G4UserStackingAction.hh"
#include "G4VModularPhysicsList.hh"
#include "G4VUserActionInitialization.hh"
#include "G4VUserDetectorConstruction.hh"
#include "G4VUserPrimaryGeneratorAction.hh"
#include "Randomize.hh"
#include "G4RunManager.hh"
#include "G4DNAChemistryManager.hh"
#include "G4ITTrackHolder.hh"
#include "G4Molecule.hh"
#include "G4MoleculeCounter.hh"
#include "G4MoleculeCounterManager.hh"
#include "G4H2O.hh"
#include "G4Scheduler.hh"
#include "G4UserRunAction.hh"
#include "G4UserTimeStepAction.hh"

namespace {

// ------------------------------------------------------------------ shared state ---------

enum class Mode { Xs, Mfp, Track, Slow, PhysChem };
Mode gMode = Mode::Xs;
G4double gCut = 0.;  // tracking cut (track mode)
G4double gEnergy = 0.;  // current primary energy

struct Accum {
  long n_events = 0;
  double sum_L = 0., sum_L2 = 0.;  // path length (nm)
  double sum_edep = 0., sum_edep2 = 0.;  // eV
  double sum_rem = 0., sum_rem2 = 0.;  // eV, kinetic energy of the primary when it was stopped
  double sum_nsec = 0., sum_nsec2 = 0., sum_esec = 0., sum_esec2 = 0.;  // secondaries (eV)
  double sum_range = 0.;  // nm, |r_end - r_start|
  long n_elastic = 0, n_excitation = 0, n_ionisation = 0, n_other = 0;
  void clear() { *this = Accum(); }
} gAcc;

// Per-event working variables
double gL = 0., gEdep = 0., gRem = 0., gNsec = 0., gEsec = 0.;
G4ThreeVector gEnd;

// Add the finished event to the accumulators (called at the end of an event, or directly from the
// stepping action in mfp mode, where the event is aborted after the first interaction).
void FinishEvent()
{
  const double L = gL / nm, E = gEdep / eV;
  ++gAcc.n_events;
  gAcc.sum_L += L;
  gAcc.sum_L2 += L * L;
  gAcc.sum_edep += E;
  gAcc.sum_edep2 += E * E;
  gAcc.sum_rem += gRem / eV;
  gAcc.sum_rem2 += (gRem / eV) * (gRem / eV);
  gAcc.sum_nsec += gNsec;
  gAcc.sum_nsec2 += gNsec * gNsec;
  gAcc.sum_esec += gEsec / eV;
  gAcc.sum_esec2 += (gEsec / eV) * (gEsec / eV);
  gAcc.sum_range += gEnd.mag() / nm;
}

void CountProcess(const G4String& name)
{
  if (name == "e-_G4DNAElastic") ++gAcc.n_elastic;
  else if (name == "e-_G4DNAExcitation") ++gAcc.n_excitation;
  else if (name == "e-_G4DNAIonisation") ++gAcc.n_ionisation;
  else ++gAcc.n_other;
}


// ------------------------------------------------------------------ slow mode (M3) --------
//
// Full slowing-down: every particle is followed to its end. Per-event quantities are summed over
// the event; per-energy sums and sums of squares are written so that the Rust test can form
// means and standard errors without access to individual events.

constexpr int kSpecBins = 70;     // 0.1 eV - 1 MeV, 10 per decade (same edges as dna-sim)
constexpr int kRadBins = 40;      // 0.1 nm - 10 mm, 5 per decade (same edges as dna-sim)
constexpr int kGenClasses = 4;    // generation 1, 2, 3, 4 or more

// Quantities summed per event, in the order of the CSV columns.
enum SlowQ {
  Q_LPRIM, Q_LALL, Q_EDEP, Q_NEL, Q_NEXC, Q_NION, Q_NVIB, Q_NATT, Q_NSOLV, Q_NSEC, Q_RANGE, Q_COUNT
};
const char* const kQNames[Q_COUNT] = {"Lprim_nm", "Lall_nm", "edep_eV", "n_elastic", "n_excitation",
                                      "n_ionisation", "n_vibexc", "n_attachment", "n_solvation",
                                      "n_secondaries", "range_nm"};

struct SlowEvent {
  double q[Q_COUNT] = {};
  long n_other_process = 0;       // steps by any other process (Transportation, ...): expected 0
  long n_other_secondary = 0;     // non-electron secondaries (killed): expected 0 with fluo/Auger off
  G4ThreeVector end;              // end point of the primary track
  std::map<int, int> generation;  // trackID -> generation (primary = 0)
};
SlowEvent gEv;

struct SlowAccum {
  long n_events = 0;
  double sum[Q_COUNT] = {};
  double sum2[Q_COUNT] = {};
  double sum_nsolv_edep = 0.;     // cross term for the solvations-per-100-eV ratio
  long n_other_process = 0, n_other_secondary = 0;
  double spec[kGenClasses][kSpecBins + 2] = {};  // [0] underflow, [1..70] bins, [71] overflow
  double rad[kRadBins + 2] = {};                 // energy (eV) summed over events, same layout
  void clear() { *this = SlowAccum(); }
} gSlow;

struct PhysChemAccum {
  long n_events = 0;
  std::map<std::string, double> sum_count, sum_count2, sum_r2_nm2;
  void clear() { *this = PhysChemAccum(); }
} gPhysChem;
std::map<std::string, double> gPhysChemEventCount, gPhysChemEventR2;
bool gChemistryRun = false;

class InitialSpeciesAction : public G4UserTimeStepAction {
 public:
  void StartProcessing() override {
    gPhysChemEventCount.clear();
    gPhysChemEventR2.clear();
    G4TrackManyList* tracks = G4ITTrackHolder::Instance()->GetMainList();
    if (tracks == nullptr) return;
    for (const G4Track* track : *tracks) {
      const auto* molecule = GetMolecule(track);
      if (molecule == nullptr || molecule->GetName() == "H2O") continue;
      const std::string name = molecule->GetName();
      ++gPhysChemEventCount[name];
      const double r = track->GetPosition().mag() / nm;
      gPhysChemEventR2[name] += r * r;
    }
    // Species are born at global time 1 ps. Stop before the first diffusion/reaction step.
    G4Scheduler::Instance()->Stop();
  }
};

std::vector<double> gSpecEdges, gRadEdges;

void BuildEdges()
{
  gSpecEdges.clear();
  gRadEdges.clear();
  for (int i = 0; i <= kSpecBins; ++i) gSpecEdges.push_back(0.1 * std::pow(10.0, i / 10.0));
  for (int i = 0; i <= kRadBins; ++i) gRadEdges.push_back(0.1 * std::pow(10.0, i / 5.0));
}

// Index in the layout [underflow, bins..., overflow] = (number of edges <= x), clipped.
int LayoutIndex(const std::vector<double>& edges, double x)
{
  const int nbins = (int)edges.size() - 1;
  const int pos = (int)(std::upper_bound(edges.begin(), edges.end(), x) - edges.begin());
  if (pos == 0) return 0;
  if (pos > nbins) return nbins + 1;
  return pos;
}

void SlowBeginEvent()
{
  gEv = SlowEvent();
}

void SlowStep(const G4Step* step)
{
  const G4Track* track = step->GetTrack();
  const G4StepPoint* post = step->GetPostStepPoint();
  const G4VProcess* proc = post->GetProcessDefinedStep();
  const G4String name = proc ? proc->GetProcessName() : G4String("none");
  const double len = step->GetStepLength() / nm;
  const double edep = step->GetTotalEnergyDeposit() / eV;

  gEv.q[Q_LALL] += len;
  if (track->GetTrackID() == 1) {
    gEv.q[Q_LPRIM] += len;
    gEv.end = post->GetPosition();
  }
  gEv.q[Q_EDEP] += edep;
  if (edep != 0.) gSlow.rad[LayoutIndex(gRadEdges, post->GetPosition().mag() / nm)] += edep;

  if (name == "e-_G4DNAElastic") gEv.q[Q_NEL] += 1;
  else if (name == "e-_G4DNAExcitation") gEv.q[Q_NEXC] += 1;
  else if (name == "e-_G4DNAIonisation") gEv.q[Q_NION] += 1;
  else if (name == "e-_G4DNAVibExcitation") gEv.q[Q_NVIB] += 1;
  else if (name == "e-_G4DNAAttachment") gEv.q[Q_NATT] += 1;
  else if (name == "e-_G4DNAElectronSolvation") gEv.q[Q_NSOLV] += 1;
  else ++gEv.n_other_process;
}

G4ClassificationOfNewTrack SlowClassify(const G4Track* t)
{
  if (t->GetParentID() == 0) {
    gEv.generation[t->GetTrackID()] = 0;
    return fUrgent;
  }
  if (t->GetDefinition() != G4Electron::Definition()) {
    ++gEv.n_other_secondary;
    return fKill;
  }
  const int g = gEv.generation[t->GetParentID()] + 1;
  gEv.generation[t->GetTrackID()] = g;
  gEv.q[Q_NSEC] += 1;
  const int cls = std::min(g, kGenClasses) - 1;
  gSlow.spec[cls][LayoutIndex(gSpecEdges, t->GetKineticEnergy() / eV)] += 1;
  return fUrgent;
}

void SlowFinishEvent()
{
  gEv.q[Q_RANGE] = gEv.end.mag() / nm;
  ++gSlow.n_events;
  for (int i = 0; i < Q_COUNT; ++i) {
    gSlow.sum[i] += gEv.q[i];
    gSlow.sum2[i] += gEv.q[i] * gEv.q[i];
  }
  gSlow.sum_nsolv_edep += gEv.q[Q_NSOLV] * gEv.q[Q_EDEP];
  gSlow.n_other_process += gEv.n_other_process;
  gSlow.n_other_secondary += gEv.n_other_secondary;
}

// ------------------------------------------------------------------ geometry -------------

class Detector : public G4VUserDetectorConstruction {
 public:
  G4VPhysicalVolume* Construct() override
  {
    auto* water = G4NistManager::Instance()->FindOrBuildMaterial("G4_WATER");
    // The sphere is the world (as in the mfp example): 1 m is far beyond any range used here.
    auto* solid = new G4Sphere("World", 0., 1. * m, 0., twopi, 0., pi);
    auto* logic = new G4LogicalVolume(solid, water, "World");
    return new G4PVPlacement(nullptr, G4ThreeVector(), logic, "World", nullptr, false, 0);
  }
};

// ------------------------------------------------------------------ physics --------------

class Physics : public G4VModularPhysicsList {
 public:
  Physics()
  {
    SetVerboseLevel(1);
    fPhysics = new G4EmDNAPhysics_option2();
    if (gMode == Mode::PhysChem) fChemistry = new G4EmDNAChemistry_option3();
  }
  ~Physics() override { delete fPhysics; delete fChemistry; }
  void ConstructParticle() override {
    fPhysics->ConstructParticle();
    if (fChemistry) fChemistry->ConstructParticle();
  }
  void ConstructProcess() override {
    AddTransportation();
    fPhysics->ConstructProcess();
    if (fChemistry) fChemistry->ConstructProcess();
  }
 private:
  G4VPhysicsConstructor* fPhysics = nullptr;
  G4VPhysicsConstructor* fChemistry = nullptr;
};

// ------------------------------------------------------------------ primary --------------

class Gun : public G4VUserPrimaryGeneratorAction {
 public:
  Gun() : fGun(new G4ParticleGun(1))
  {
    fGun->SetParticleDefinition(G4Electron::Definition());
    // along +z like the Rust runs (RunConfig default); the elastic model of Geant4 depends
    // (slightly) on the direction through CLHEP orthogonal(), see docs/09 Q13
    fGun->SetParticleMomentumDirection(G4ThreeVector(0., 0., 1.));
    fGun->SetParticlePosition(G4ThreeVector());
  }
  ~Gun() override { delete fGun; }
  void GeneratePrimaries(G4Event* ev) override
  {
    fGun->SetParticleEnergy(gEnergy);
    fGun->GeneratePrimaryVertex(ev);
  }

 private:
  G4ParticleGun* fGun;
};

// ------------------------------------------------------------------ stepping -------------

class Stepping : public G4UserSteppingAction {
 public:
  void UserSteppingAction(const G4Step* step) override
  {
    if (gMode == Mode::Slow) {
      SlowStep(step);
      return;
    }
    G4Track* track = step->GetTrack();
    if (track->GetTrackID() != 1) return;  // secondaries are killed in the stacking action
    const G4StepPoint* pre = step->GetPreStepPoint();
    const G4StepPoint* post = step->GetPostStepPoint();
    const G4String proc = post->GetProcessDefinedStep()->GetProcessName();

    if (gMode == Mode::Mfp) {
      // first interaction only: the free path is the first step length
      if (proc == "Transportation") return;  // left the sphere: not a sample (never expected)
      gL = step->GetStepLength();
      gEnd = post->GetPosition();
      CountProcess(proc);
      FinishEvent();
      G4RunManager::GetRunManager()->AbortEvent();
      return;
    }

    // track mode: count the step only if the primary was at or above the cut at its start
    if (pre->GetKineticEnergy() < gCut) {
      track->SetTrackStatus(fStopAndKill);
      return;
    }
    gL += step->GetStepLength();
    gEdep += step->GetTotalEnergyDeposit();
    gEnd = post->GetPosition();
    CountProcess(proc);
    if (post->GetKineticEnergy() < gCut) {
      gRem = post->GetKineticEnergy();
      track->SetTrackStatus(fStopAndKill);  // remaining energy is *not* deposited: it is `gRem`
    }
  }
};

class Stacking : public G4UserStackingAction {
 public:
  void NewStage() override
  {
    if (gMode == Mode::PhysChem && !gChemistryRun && stackManager->GetNTotalTrack() == 0) {
      gChemistryRun = true;
      G4DNAChemistryManager::Instance()->Run();
    }
  }
  G4ClassificationOfNewTrack ClassifyNewTrack(const G4Track* t) override
  {
    if (gMode == Mode::Slow) return SlowClassify(t);
    if (t->GetParentID() == 0) return fUrgent;
    // every secondary: record, then kill (geant4-rs M2 records secondaries without transporting)
    if (gMode == Mode::Track) {
      ++gNsec;
      gEsec += t->GetKineticEnergy();
    }
    return fKill;
  }
};

class EventAct : public G4UserEventAction {
 public:
  void BeginOfEventAction(const G4Event*) override
  {
    gL = gEdep = gRem = gNsec = gEsec = 0.;
    gEnd = G4ThreeVector();
    if (gMode == Mode::Slow) SlowBeginEvent();
    if (gMode == Mode::PhysChem) {
      gChemistryRun = false;
      gPhysChemEventCount.clear();
      gPhysChemEventR2.clear();
    }
  }
  void EndOfEventAction(const G4Event*) override
  {
    if (gMode == Mode::Track) FinishEvent();  // mfp mode finishes inside the stepping action
    if (gMode == Mode::Slow) SlowFinishEvent();
    if (gMode == Mode::PhysChem) {
      auto* counter = G4MoleculeCounterManager::Instance()->GetMoleculeCounter<G4MoleculeCounter>(0);
      if (counter != nullptr) {
        for (const auto& index : counter->GetMapIndices()) {
          const double count = counter->GetNbMoleculesAtTime(index, 1. * ps);
          if (count > 0.) gPhysChemEventCount[index.Molecule->GetName()] = count;
        }
      }
      for (const auto& item : gPhysChemEventCount) {
        gPhysChem.sum_count[item.first] += item.second;
        gPhysChem.sum_count2[item.first] += item.second * item.second;
        gPhysChem.sum_r2_nm2[item.first] += gPhysChemEventR2[item.first];
      }
      ++gPhysChem.n_events;
    }
  }
};

class RunAct : public G4UserRunAction {
 public:
  void BeginOfRunAction(const G4Run* run) override {
    if (auto* manager = G4DNAChemistryManager::GetInstanceIfExists()) manager->BeginOfRunAction(run);
  }
  void EndOfRunAction(const G4Run* run) override {
    if (auto* manager = G4DNAChemistryManager::GetInstanceIfExists()) manager->EndOfRunAction(run);
  }
};

class Actions : public G4VUserActionInitialization {
 public:
  void Build() const override
  {
    SetUserAction(new Gun);
    SetUserAction(new Stepping);
    SetUserAction(new Stacking);
    SetUserAction(new EventAct);
    SetUserAction(new RunAct);
    if (gMode == Mode::PhysChem) {
      auto* counters = G4MoleculeCounterManager::Instance();
      counters->SetResetCountersBeforeEvent(true);
      counters->SetResetCountersBeforeRun(true);
      counters->SetAccumulateCounterIntoMaster(false);
      auto counter = std::make_unique<G4MoleculeCounter>();
      counter->SetTimeComparer(G4MoleculeCounterTimeComparer::CreateWithFixedPrecision(1. * ps));
      counter->IgnoreMolecule(G4H2O::Definition());
      counters->RegisterCounter(std::move(counter));
      // InitialSpeciesAction is installed in main after scheduler initialisation.
    }
  }
};

// ------------------------------------------------------------------ output ---------------

void WriteProvenance(FILE* f, const char* mode, long events, long seed)
{
  fprintf(f, "# generator: g4rs_ref (validation/geant4/g4rs_ref), mode %s\n", mode);
  fprintf(f, "# geant4_version_number: %d (%s)\n", G4VERSION_NUMBER, G4Version.c_str());
  fprintf(f, "# physics: G4EmDNAPhysics_option2; material G4_WATER; fluo/auger OFF; events %ld; seed %ld\n", events, seed);
}

void RunXs(const char* out)
{
  G4EmCalculator calc;
  FILE* f = fopen(out, "w");
  WriteProvenance(f, "xs", 0, 0);
  fprintf(f, "energy_eV,process,sigma_per_volume_per_nm\n");
  std::vector<double> energies = {7.4, 9., 11., 100., 1e3, 1e4, 1e5, 1e6};
  const int n = 200;
  for (int i = 0; i < n; ++i) energies.push_back(7.4 * std::pow(1e6 / 7.4, i / (n - 1.0)));
  const char* procs[3][2] = {{"elastic", "e-_G4DNAElastic"},
                              {"excitation", "e-_G4DNAExcitation"},
                              {"ionisation", "e-_G4DNAIonisation"}};
  for (double e : energies) {
    for (auto& p : procs) {
      double s = calc.ComputeCrossSectionPerVolume(e * eV, "e-", p[1], "G4_WATER", 0.);
      fprintf(f, "%.17g,%s,%.17g\n", e, p[0], s * nm);  // 1/mm -> 1/nm
    }
  }
  fclose(f);
}

// Cross sections of the five processes below 150 eV, for the V2 comparison of the M3 models.
void RunXsLow(const char* out)
{
  G4EmCalculator calc;
  FILE* f = fopen(out, "w");
  WriteProvenance(f, "xslow", 0, 0);
  fprintf(f, "energy_eV,process,sigma_per_volume_per_nm\n");
  // window boundaries, the Melton table end and its neighbours, the Sanche edge, and values just
  // outside the windows (expected 0), then 200 log-spaced energies from 2 eV to 100 eV
  std::vector<double> energies = {1.5, 2., 3.9, 4., 7.4, 9., 11., 12.3, 12.8, 13., 13.5, 100., 101., 150.};
  const int n = 200;
  for (int i = 0; i < n; ++i) energies.push_back(2. * std::pow(100. / 2., i / (n - 1.0)));
  const char* procs[5][2] = {{"elastic", "e-_G4DNAElastic"},
                              {"excitation", "e-_G4DNAExcitation"},
                              {"ionisation", "e-_G4DNAIonisation"},
                              {"vibexc", "e-_G4DNAVibExcitation"},
                              {"attachment", "e-_G4DNAAttachment"}};
  for (double e : energies) {
    for (auto& p : procs) {
      double s = calc.ComputeCrossSectionPerVolume(e * eV, "e-", p[1], "G4_WATER", 0.);
      fprintf(f, "%.17g,%s,%.17g\n", e, p[0], s * nm);
    }
  }
  fclose(f);
}

void RunEvents(const char* out, const char* mode, std::vector<double> energies, long events, long seed)
{
  FILE* f = fopen(out, "w");
  WriteProvenance(f, mode, events, seed);
  if (gMode == Mode::Track) fprintf(f, "# tracking cut: %.17g eV\n", gCut / eV);
  fprintf(f,
          "energy_eV,n_events,sum_L_nm,sum_L2_nm2,sum_edep_eV,sum_edep2_eV2,sum_remaining_eV,"
          "sum_remaining2_eV2,sum_nsec,sum_nsec2,sum_esec_eV,sum_esec2_eV2,sum_range_nm,n_elastic,"
          "n_excitation,n_ionisation,n_other\n");
  auto* rm = G4RunManager::GetRunManager();
  for (double e : energies) {
    gEnergy = e * eV;
    gAcc.clear();
    rm->BeamOn((G4int)events);
    fprintf(f, "%.17g,%ld,%.17g,%.17g,%.17g,%.17g,%.17g,%.17g,%.17g,%.17g,%.17g,%.17g,%.17g,%ld,%ld,%ld,%ld\n",
            e, gAcc.n_events, gAcc.sum_L, gAcc.sum_L2, gAcc.sum_edep, gAcc.sum_edep2, gAcc.sum_rem,
            gAcc.sum_rem2, gAcc.sum_nsec, gAcc.sum_nsec2, gAcc.sum_esec, gAcc.sum_esec2,
            gAcc.sum_range, gAcc.n_elastic, gAcc.n_excitation, gAcc.n_ionisation, gAcc.n_other);
    fflush(f);
  }
  fclose(f);
}


// "8:100000,30:100000,..." -> list of (energy eV, events)
std::vector<std::pair<double, long>> ParseSpec(const std::string& spec)
{
  std::vector<std::pair<double, long>> out;
  size_t start = 0;
  while (start < spec.size()) {
    size_t comma = spec.find(',', start);
    if (comma == std::string::npos) comma = spec.size();
    const std::string item = spec.substr(start, comma - start);
    const size_t colon = item.find(':');
    if (colon == std::string::npos) {
      fprintf(stderr, "bad energy:events item `%s`\n", item.c_str());
      exit(1);
    }
    out.emplace_back(atof(item.substr(0, colon).c_str()), atol(item.substr(colon + 1).c_str()));
    start = comma + 1;
  }
  return out;
}

void RunSlow(const std::string& prefix, long seed, const std::vector<std::pair<double, long>>& cases)
{
  BuildEdges();
  FILE* fs = fopen((prefix + "_sums.csv").c_str(), "w");
  FILE* fp = fopen((prefix + "_spectra.csv").c_str(), "w");
  FILE* fr = fopen((prefix + "_radial.csv").c_str(), "w");
  if (!fs || !fp || !fr) {
    fprintf(stderr, "cannot open output files with prefix %s\n", prefix.c_str());
    exit(1);
  }
  for (FILE* f : {fs, fp, fr}) {
    WriteProvenance(f, "slow", 0, seed);
    fprintf(f, "# full slowing-down: primary and all secondaries followed to solvation/attachment, nothing killed or cut\n");
  }
  fprintf(fs, "# per-event quantities are summed over the event; columns sum_<q> and sum2_<q> (sum of squares) per quantity\n");
  fprintf(fs, "energy_eV,n_events");
  for (int i = 0; i < Q_COUNT; ++i) fprintf(fs, ",sum_%s,sum2_%s", kQNames[i], kQNames[i]);
  fprintf(fs, ",sum_nsolv_x_edep,n_other_process,n_other_secondary\n");
  fprintf(fp, "# bin 0 = underflow (< 0.1 eV), bins 1..%d: edge_{k-1} <= E < edge_k with edge_k = 0.1 eV x 10^(k/10), bin %d = overflow (>= 1 MeV)\n", kSpecBins, kSpecBins + 1);
  fprintf(fp, "# generation_class 1 = ejected by the primary, 2, 3, 4 = fourth generation or later\n");
  fprintf(fp, "energy_eV,generation_class,bin,count\n");
  fprintf(fr, "# bin 0 = underflow (< 0.1 nm), bins 1..%d: edge_{k-1} <= r < edge_k with edge_k = 0.1 nm x 10^(k/5), bin %d = overflow (>= 10 mm)\n", kRadBins, kRadBins + 1);
  fprintf(fr, "# energy_eV_sum = energy deposited in the shell summed over all events of this primary energy\n");
  fprintf(fr, "energy_eV,bin,energy_eV_sum\n");

  auto* rm = G4RunManager::GetRunManager();
  for (const auto& c : cases) {
    gEnergy = c.first * eV;
    gSlow.clear();
    rm->BeamOn((G4int)c.second);
    fprintf(fs, "%.17g,%ld", c.first, gSlow.n_events);
    for (int i = 0; i < Q_COUNT; ++i) fprintf(fs, ",%.17g,%.17g", gSlow.sum[i], gSlow.sum2[i]);
    fprintf(fs, ",%.17g,%ld,%ld\n", gSlow.sum_nsolv_edep, gSlow.n_other_process, gSlow.n_other_secondary);
    for (int g = 0; g < kGenClasses; ++g)
      for (int b = 0; b < kSpecBins + 2; ++b)
        fprintf(fp, "%.17g,%d,%d,%.0f\n", c.first, g + 1, b, gSlow.spec[g][b]);
    for (int b = 0; b < kRadBins + 2; ++b) fprintf(fr, "%.17g,%d,%.17g\n", c.first, b, gSlow.rad[b]);
    fflush(fs);
    fflush(fp);
    fflush(fr);
    fprintf(stderr, "slow: %.17g eV done (%ld events)\n", c.first, gSlow.n_events);
  }
  fclose(fs);
  fclose(fp);
  fclose(fr);
}

void RunPhysChem(const char* out, long seed, const std::vector<std::pair<double, long>>& cases)
{
  FILE* f = fopen(out, "w");
  if (!f) { fprintf(stderr, "cannot open %s\n", out); exit(1); }
  WriteProvenance(f, "physchem", 0, seed);
  fprintf(f, "# G4EmDNAChemistry_option3; stopped in StartProcessing at 1 ps before diffusion/reactions\n");
  fprintf(f, "energy_eV,n_events,species,sum_count,sum_count2,sum_r2_nm2\n");
  auto* rm = G4RunManager::GetRunManager();
  for (const auto& c : cases) {
    gEnergy = c.first * eV;
    gPhysChem.clear();
    rm->BeamOn((G4int)c.second);
    for (const auto& item : gPhysChem.sum_count) {
      const std::string& species = item.first;
      fprintf(f, "%.17g,%ld,%s,%.17g,%.17g,%.17g\n", c.first, gPhysChem.n_events,
              species.c_str(), item.second, gPhysChem.sum_count2[species],
              gPhysChem.sum_r2_nm2[species]);
    }
    fflush(f);
    fprintf(stderr, "physchem: %.17g eV done (%ld events)\n", c.first, gPhysChem.n_events);
  }
  fclose(f);
}

}  // namespace

int main(int argc, char** argv)
{
  if (argc < 3) {
    fprintf(stderr, "usage: %s xs <out.csv> | mfp <out.csv> <events> <seed> | track <out.csv> <events> <seed> <cut_eV> | slow <out_prefix> <seed> [E_eV:events,...] | physchem <out.csv> <seed> [E_eV:events,...]\n", argv[0]);
    return 1;
  }
  const std::string mode = argv[1];
  const char* out = argv[2];
  long events = argc > 3 ? atol(argv[3]) : 0;
  long seed = argc > 4 ? atol(argv[4]) : 1;
  std::string slowSpec =
      "8:100000,30:100000,100:100000,1000:20000,10000:2000,100000:200,900000:20";
  bool xslow = false;
  if (mode == "xslow") xslow = true;
  if (mode == "slow") {
    // slow <out_prefix> <seed> [spec]: the seed is argv[3] here
    seed = argc > 3 ? atol(argv[3]) : 1;
    if (argc > 4) slowSpec = argv[4];
    gMode = Mode::Slow;
  }
  else if (mode == "physchem") {
    seed = argc > 3 ? atol(argv[3]) : 1;
    slowSpec = argc > 4 ? argv[4] : "1000:20000,10000:2000,100000:200,900000:20";
    gMode = Mode::PhysChem;
  }
  else if (mode == "mfp") gMode = Mode::Mfp;
  else if (mode == "track") {
    gMode = Mode::Track;
    gCut = (argc > 5 ? atof(argv[5]) : 100.) * eV;
  }
  else gMode = Mode::Xs;

  auto* rm = G4RunManagerFactory::CreateRunManager(G4RunManagerType::Serial);
  rm->SetUserInitialization(new Detector);
  rm->SetUserInitialization(new Physics);
  rm->SetUserInitialization(new Actions);

  // Atomic de-excitation off (see header). Must be set before initialisation.
  auto* param = G4EmParameters::Instance();
  param->SetFluo(false);
  param->SetAuger(false);
  param->SetAugerCascade(false);
  param->SetPixe(false);

  G4Random::setTheSeed(seed);
  rm->Initialize();
  // EndTime must exceed the 1 ps species birth time for StartProcessing to be entered; the user
  // action records the initial list and calls Stop() before the first chemistry step.
  if (gMode == Mode::PhysChem) {
    G4Scheduler::Instance()->SetEndTime(2. * ps);
    G4Scheduler::Instance()->SetUserAction(new InitialSpeciesAction());
  }

  // SetFluo(false) alone is not enough with the DNA ionisation models (11.4.1): G4EmDNABuilder
  // always registers a G4UAtomicDeexcitation, and Born2/Emfietzoglou call GetAtomicShell() and
  // GenerateParticles() on it for K-shell ionisation without checking IsFluoActive(). With fluo
  // off its transition tables are not loaded -> fatal exception de0001; with the tables loaded it
  // would emit fluorescence/Auger. Dropping it (models read it at BeamOn) gives "no
  // de-excitation": the K-shell binding energy stays local, as in geant4-rs M2.
  if (gMode != Mode::PhysChem) G4LossTableManager::Instance()->SetAtomDeexcitation(nullptr);

  if (gMode == Mode::Xs) {
    // G4EmCalculator needs the physics tables: BeamOn(0) runs RunInitialization without events
    rm->BeamOn(0);
    if (xslow) RunXsLow(out);
    else RunXs(out);
  }
  else if (gMode == Mode::Mfp) {
    // 100 eV is excluded: the Sanche window [2, 100] eV is closed at 100 eV. 1 MeV is excluded:
    // standard eIoni is also active exactly at 1 MeV. Keep this list identical to the Rust test.
    RunEvents(out, "mfp", {150., 300., 1e3, 3e3, 1e4, 3e4, 1e5, 3e5, 9e5}, events, seed);
  }
  else if (gMode == Mode::Slow) {
    RunSlow(out, seed, ParseSpec(slowSpec));
  }
  else if (gMode == Mode::PhysChem) {
    RunPhysChem(out, seed, ParseSpec(slowSpec));
  }
  else {
    RunEvents(out, "track", {200., 1e3, 1e4, 1e5, 9e5}, events, seed);
  }
  delete rm;
  return 0;
}
