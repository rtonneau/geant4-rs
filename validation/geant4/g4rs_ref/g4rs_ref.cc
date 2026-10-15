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
//   mfp   <out.csv> <events> <seed>       first interaction only, like examples/.../mfp (V5)
//   track <out.csv> <events> <seed> <cut_eV>
//                                         primary electron followed until its kinetic energy falls
//                                         below <cut_eV>; secondaries are killed (their energy is
//                                         summed); like examples/.../range with a tracking cut (V5)

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
#include "G4SystemOfUnits.hh"
#include "G4Track.hh"
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

namespace {

// ------------------------------------------------------------------ shared state ---------

enum class Mode { Xs, Mfp, Track };
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
    RegisterPhysics(new G4EmDNAPhysics_option2());
    RegisterPhysics(new G4DecayPhysics());
  }
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
  G4ClassificationOfNewTrack ClassifyNewTrack(const G4Track* t) override
  {
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
  }
  void EndOfEventAction(const G4Event*) override
  {
    if (gMode == Mode::Track) FinishEvent();  // mfp mode finishes inside the stepping action
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

}  // namespace

int main(int argc, char** argv)
{
  if (argc < 3) {
    fprintf(stderr, "usage: %s xs <out.csv> | mfp <out.csv> <events> <seed> | track <out.csv> <events> <seed> <cut_eV>\n", argv[0]);
    return 1;
  }
  const std::string mode = argv[1];
  const char* out = argv[2];
  long events = argc > 3 ? atol(argv[3]) : 0;
  long seed = argc > 4 ? atol(argv[4]) : 1;
  if (mode == "mfp") gMode = Mode::Mfp;
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

  // SetFluo(false) alone is not enough with the DNA ionisation models (11.4.1): G4EmDNABuilder
  // always registers a G4UAtomicDeexcitation, and Born2/Emfietzoglou call GetAtomicShell() and
  // GenerateParticles() on it for K-shell ionisation without checking IsFluoActive(). With fluo
  // off its transition tables are not loaded -> fatal exception de0001; with the tables loaded it
  // would emit fluorescence/Auger. Dropping it (models read it at BeamOn) gives "no
  // de-excitation": the K-shell binding energy stays local, as in geant4-rs M2.
  G4LossTableManager::Instance()->SetAtomDeexcitation(nullptr);

  if (gMode == Mode::Xs) {
    // G4EmCalculator needs the physics tables: BeamOn(0) runs RunInitialization without events
    rm->BeamOn(0);
    RunXs(out);
  }
  else if (gMode == Mode::Mfp) {
    // 100 eV is excluded: the Sanche window [2, 100] eV is closed at 100 eV. 1 MeV is excluded:
    // standard eIoni is also active exactly at 1 MeV. Keep this list identical to the Rust test.
    RunEvents(out, "mfp", {150., 300., 1e3, 3e3, 1e4, 3e4, 1e5, 3e5, 9e5}, events, seed);
  }
  else {
    RunEvents(out, "track", {200., 1e3, 1e4, 1e5, 9e5}, events, seed);
  }
  delete rm;
  return 0;
}
