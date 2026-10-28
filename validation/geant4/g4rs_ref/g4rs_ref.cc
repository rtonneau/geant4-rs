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
//   physchem_prechem <out.csv> <seed> <E_eV:events,...>
//                                         M6 diagnostic: snapshot after all water decays, in the
//                                         zero-step post-callback before IRT initialization.
//                                         Existing physchem mode and its output are unchanged.
//   physchem_spatial_diagnostic <out.csv> <histories_per_seed> <seed,...>
//                                         M6 five-boundary long-form diagnostic at 900 keV.
//   two_body_h_h <out.csv> <seed> <pairs>
//                                         M7: isolated option3 H+H SBS survival curve from
//                                         r0=0.2 nm at the six pre-registered tau values.

#include <algorithm>
#include <cerrno>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <map>
#include <limits>
#include <set>
#include <sstream>
#include <string>
#include <vector>

#include "G4DNAIRT.hh"
#include "CLHEP/Random/RanecuEngine.h"
#include "G4DecayPhysics.hh"
#include "G4EmCalculator.hh"
#include "G4EmDNAPhysics_option2.hh"
#include "G4EmDNAChemistry_option3.hh"
#include "G4EmParameters.hh"
#include "G4Electron.hh"
#include "G4Event.hh"
#include "G4Geantino.hh"
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
#include "G4ITGun.hh"
#include "G4Molecule.hh"
#include "G4MoleculeTable.hh"
#include "G4MoleculeCounter.hh"
#include "G4MoleculeCounterManager.hh"
#include "G4H2O.hh"
#include "G4ElectronOccupancy.hh"
#include "G4MoleculeDefinition.hh"
#include "G4Scheduler.hh"
#include "G4UserRunAction.hh"
#include "G4UserTimeStepAction.hh"

namespace {

// ------------------------------------------------------------------ shared state ---------

enum class Mode { Xs, Mfp, Track, Slow, PhysChem, PreChem, SpatialDiagnostic, TwoBodyHH };
Mode gMode = Mode::Xs;
bool IsChemistryMode() { return gMode == Mode::PhysChem || gMode == Mode::PreChem ||
                                gMode == Mode::SpatialDiagnostic || gMode == Mode::TwoBodyHH; }
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
bool gPreChemCaptured = false;

// Isolated H+H reference state. TIME_SCALE_PS is the independently registered V9 value
// (r0-sigma)^2/(4*D_rel), with option3 reaction id 1 and r0=0.2 nm.
constexpr G4double kTwoBodyTau[] = {0.25, 0.5, 1., 2., 4., 16.};
constexpr G4double kTwoBodyTimeScalePs = 0.19705088916617716;
constexpr size_t kTwoBodyPoints = sizeof(kTwoBodyTau) / sizeof(kTwoBodyTau[0]);
long gTwoBodyCompleted = 0;
long gTwoBodyReacted[kTwoBodyPoints] = {};
long gTwoBodySurviving[kTwoBodyPoints] = {};
bool gTwoBodyDidReact = false;
size_t gTwoBodyNextPoint = 0;

[[noreturn]] void TwoBodyFailure(const char* message)
{
  fprintf(stderr, "two_body_h_h: %s; output is incomplete, do not commit it\n", message);
  exit(1);
}

class TwoBodyGun : public G4ITGun {
 public:
  void DefineTracks() override
  {
    auto* configuration = G4MoleculeTable::Instance()->GetConfiguration("H");
    if (!configuration) TwoBodyFailure("option3 H configuration is unavailable");
    PushTrack((new G4Molecule(configuration))->BuildTrack(0., G4ThreeVector(-0.1 * nm, 0., 0.)));
    PushTrack((new G4Molecule(configuration))->BuildTrack(0., G4ThreeVector( 0.1 * nm, 0., 0.)));
  }
};

class TwoBodyAction : public G4UserTimeStepAction {
 public:
  void StartProcessing() override
  {
    gTwoBodyDidReact = false;
    gTwoBodyNextPoint = 0;
  }
  void UserReactionAction(const G4Track& trackA, const G4Track& trackB,
                          const std::vector<G4Track*>*) override
  {
    const auto* a = GetMolecule(&trackA);
    const auto* b = GetMolecule(&trackB);
    if (a && b && a->GetName() == "H^0" && b->GetName() == "H^0") gTwoBodyDidReact = true;
  }
  void UserPostTimeStepAction() override
  {
    const G4double now = G4Scheduler::Instance()->GetGlobalTime() / ps;
    while (gTwoBodyNextPoint < kTwoBodyPoints &&
           now + 16. * std::numeric_limits<G4double>::epsilon() * std::max(1., now) >=
               kTwoBodyTau[gTwoBodyNextPoint] * kTwoBodyTimeScalePs) {
      if (gTwoBodyDidReact) ++gTwoBodyReacted[gTwoBodyNextPoint];
      else ++gTwoBodySurviving[gTwoBodyNextPoint];
      ++gTwoBodyNextPoint;
    }
  }
  void EndProcessing() override
  {
    if (gTwoBodyNextPoint != kTwoBodyPoints)
      TwoBodyFailure("scheduler ended before every observation was recorded");
    ++gTwoBodyCompleted;
  }
};

// Long-form spatial diagnostic state. It is deliberately isolated from the historical
// physchem modes so their arguments and bytes remain unchanged.
FILE* gSpatialFile = nullptr;
long gSpatialSeed = 0;
long gSpatialHistory = -1;
unsigned long gSpatialSourceOrdinal = 0;
unsigned long gSpatialPass = 0;
unsigned long gSpatialProcessingOrdinal = 0;
struct PhysicalSite {
  long track = 0, parent = 0;
  G4ThreeVector origin;
  std::string process;
  unsigned long source = 0;
  bool assigned = false;
};
std::vector<PhysicalSite> gSpatialSites;
std::set<long> gSpatialKnownMolecules;
std::map<long, unsigned long> gSpatialMoleculeSource;
std::map<long, long> gSpatialMoleculePhysicalTrack;
std::set<std::pair<std::string, std::string>> gSpatialPopulated;

[[noreturn]] void SpatialFailure(const char* message)
{
  fprintf(stderr, "physchem_spatial_diagnostic: %s; output is incomplete, do not commit it\n",
          message);
  exit(1);
}

const char* kSpatialHeader =
    "implementation,seed,history,boundary,category,entity_namespace,entity_id,"
    "physical_track_id,parent_namespace,parent_id,source_event,creation_ordinal,"
    "zero_time_pass,processing_ordinal,stage,time_ps,state,count,sum_r2_nm2,sum_z_nm,"
    "sum_z2_nm2,sum_x2_plus_y2_nm2,max_radius_nm,radial_bin_0,radial_bin_1,radial_bin_2,"
    "radial_bin_3,radial_bin_4,radial_bin_5,radial_bin_6,radial_bin_7,radial_bin_8,"
    "radial_bin_9,radial_bin_10,radial_bin_11,radial_bin_12,radial_bin_13,radial_bin_14,"
    "radial_bin_15,radial_bin_16,radial_bin_17";

int SpatialRadialBin(double r_nm)
{
  int bin = 0;
  while (bin < 17 && r_nm >= std::pow(10., bin / 2.)) ++bin;
  return bin;
}

void SpatialRow(const char* boundary, const std::string& category, const char* entityNamespace,
                long entityId, const char* physicalTrack, const char* parentNamespace,
                const char* parentId, const char* sourceEvent, unsigned long creationOrdinal,
                const char* zeroPass, const char* processingOrdinal, const char* stage,
                const char* state, const G4ThreeVector& position)
{
  if (!gSpatialFile) SpatialFailure("row emitted without an open output");
  const double x = position.x() / nm, y = position.y() / nm, z = position.z() / nm;
  const double transverse = x * x + y * y, z2 = z * z, r2 = transverse + z2;
  const double radius = std::sqrt(r2);
  fprintf(gSpatialFile,
          "geant4,%ld,%ld,%s,%s,%s,%ld,%s,%s,%s,%s,%lu,%s,%s,%s,%s,%s,1,"
          "%.17g,%.17g,%.17g,%.17g,%.17g",
          gSpatialSeed, gSpatialHistory, boundary, category.c_str(), entityNamespace, entityId,
          physicalTrack, parentNamespace, parentId, sourceEvent, creationOrdinal, zeroPass,
          processingOrdinal, stage, strcmp(stage, "physical") == 0 ? "NA" : "1", state,
          r2, z, z2, transverse, radius);
  const int occupied = SpatialRadialBin(radius);
  for (int bin = 0; bin < 18; ++bin) fprintf(gSpatialFile, ",%d", bin == occupied ? 1 : 0);
  fprintf(gSpatialFile, "\n");
  gSpatialPopulated.insert({boundary, category});
}

void SpatialEmpty(const char* boundary, const std::string& category, const char* stage)
{
  if (gSpatialPopulated.count({boundary, category})) return;
  fprintf(gSpatialFile, "geant4,%ld,%ld,%s,%s,NA,NA,NA,NA,NA,NA,NA,NA,NA,%s,NA,NA,0,"
                        "0,0,0,0,0", gSpatialSeed, gSpatialHistory, boundary,
          category.c_str(), stage);
  for (int bin = 0; bin < 18; ++bin) fprintf(gSpatialFile, ",0");
  fprintf(gSpatialFile, "\n");
}

std::string SpatialSpecies(const G4Molecule* molecule)
{
  const std::string name = molecule->GetName();
  if (name == "H3O^1") return "H3Op";
  if (name == std::string("\xC2\xB0") + "OH^0") return std::string("\xC2\xB0") + "OH";
  if (name == "OH^-1") return "OHm";
  if (name == "e_aq^-1") return "e_aq";
  if (name == "H^0") return "H";
  if (name == "H_2^0") return "H2";
  if (name == std::string("\xC2\xB0") + "O^0") return "Oxy";
  return "";
}

std::string SpatialWaterCategory(const G4Molecule* molecule)
{
  const auto* occupancy = molecule->GetElectronOccupancy();
  const auto* ground = molecule->GetDefinition()->GetGroundStateElectronOccupancy();
  if (!occupancy || !ground) SpatialFailure("water state lacks electron occupancy");
  const int delta = occupancy->GetTotalOccupancy() - ground->GetTotalOccupancy();
  if (delta == 1) return "attachment";
  for (int orbit = 0; orbit < 5; ++orbit) {
    if (occupancy->GetOccupancy(orbit) < ground->GetOccupancy(orbit)) {
      const int level = 4 - orbit;
      return std::string(delta == 0 ? "excitation_" : "ionisation_") +
             std::to_string(level);
    }
  }
  SpatialFailure("unrecognized pre-decay water state");
}

PhysicalSite& SpatialAssignSite(long parent, const std::string& category)
{
  const bool solvation = category == "solvation";
  const bool attachment = category == "attachment";
  const bool excitation = category.rfind("excitation_", 0) == 0;
  const bool ionisation = category.rfind("ionisation_", 0) == 0;
  for (auto& site : gSpatialSites) {
    if (site.assigned || site.track != parent) continue;
    const bool matches = (solvation && site.process == "e-_G4DNAElectronSolvation") ||
                         (attachment && site.process == "e-_G4DNAAttachment") ||
                         (excitation && site.process == "e-_G4DNAExcitation") ||
                         (ionisation && site.process == "e-_G4DNAIonisation");
    if (matches) { site.assigned = true; return site; }
  }
  SpatialFailure("molecular insertion could not be matched to its physical site");
}

void SpatialPhysicalRow(PhysicalSite& site, const std::string& category, const char* state)
{
  const std::string track = std::to_string(site.track);
  const std::string parent = site.parent == 0 ? "NA" : std::to_string(site.parent);
  const std::string source = std::to_string(site.source);
  SpatialRow("physical_sites", category, "event", (long)site.source, track.c_str(),
             site.parent == 0 ? "NA" : "physical", parent.c_str(), source.c_str(), site.source,
             "NA", "NA", "physical", state, site.origin);
}

void SpatialInitialMolecule(const G4Track* track)
{
  const auto* molecule = GetMolecule(track);
  if (!molecule) SpatialFailure("initial track is not molecular");
  const long id = track->GetTrackID(), parent = track->GetParentID();
  const std::string species = SpatialSpecies(molecule);
  const bool electron = species == "e_aq";
  const std::string category = electron ? "solvation" : SpatialWaterCategory(molecule);
  auto& site = SpatialAssignSite(parent, category);
  SpatialPhysicalRow(site, category, electron ? "killed" :
                    (category == "attachment" ? "killed" : "alive"));
  const std::string entity = std::to_string(id), physical = std::to_string(parent);
  const std::string source = std::to_string(site.source);
  const unsigned long creation = (unsigned long)(-id - 1);
  SpatialRow(electron ? "solvated_electron_creation" : "pre_decay_water",
             electron ? species : category, "molecular", id, physical.c_str(), "physical",
             physical.c_str(), source.c_str(), creation, "NA", "NA", "physicochemical",
             electron ? "alive" : "stopped", track->GetPosition());
  gSpatialKnownMolecules.insert(id);
  gSpatialMoleculeSource[id] = site.source;
  gSpatialMoleculePhysicalTrack[id] = parent;
}

void SpatialDaughter(const G4Track* track, unsigned long processingOrdinal)
{
  const auto* molecule = GetMolecule(track);
  const std::string species = molecule ? SpatialSpecies(molecule) : "";
  if (species.empty()) return;  // H2Ovib is an intermediate, not a frozen population.
  const long id = track->GetTrackID(), parent = track->GetParentID();
  const auto sourceIt = gSpatialMoleculeSource.find(parent);
  const auto physicalIt = gSpatialMoleculePhysicalTrack.find(parent);
  if (sourceIt == gSpatialMoleculeSource.end() || physicalIt == gSpatialMoleculePhysicalTrack.end())
    SpatialFailure("daughter lineage does not reach an initial physical site");
  gSpatialMoleculeSource[id] = sourceIt->second;
  gSpatialMoleculePhysicalTrack[id] = physicalIt->second;
  const std::string entity = std::to_string(id), parentId = std::to_string(parent);
  const std::string physical = std::to_string(physicalIt->second);
  const std::string source = std::to_string(sourceIt->second);
  const unsigned long creation = (unsigned long)(-id - 1);
  const std::string pass = std::to_string(gSpatialPass);
  const std::string processing = std::to_string(processingOrdinal);
  SpatialRow("dissociation_products", species, "molecular", id, physical.c_str(), "molecular",
             parentId.c_str(), source.c_str(), creation, pass.c_str(), processing.c_str(),
             "physicochemical", "alive", track->GetPosition());
}

void SpatialCompleteHistory(G4TrackManyList* tracks)
{
  static const char* species[] = {"e_aq", "\xC2\xB0" "OH", "H3Op", "H", "H2", "OHm", "Oxy"};
  for (const G4Track* track : *tracks) {
    const auto* molecule = GetMolecule(track);
    const std::string name = molecule ? SpatialSpecies(molecule) : "";
    if (name.empty() || track->GetTrackStatus() != fAlive) continue;
    const std::string parent = std::to_string(track->GetParentID());
    const auto sourceIt = gSpatialMoleculeSource.find(track->GetTrackID());
    const auto physicalIt = gSpatialMoleculePhysicalTrack.find(track->GetTrackID());
    if (sourceIt == gSpatialMoleculeSource.end() || physicalIt == gSpatialMoleculePhysicalTrack.end())
      SpatialFailure("survivor lineage does not reach an initial physical site");
    const std::string source = std::to_string(sourceIt->second);
    const std::string physical = std::to_string(physicalIt->second);
    SpatialRow("final_survivors", name, "molecular", track->GetTrackID(), physical.c_str(),
               track->GetParentID() > 0 ? "physical" : "molecular", parent.c_str(), source.c_str(),
               (unsigned long)(-track->GetTrackID() - 1), "NA", "NA",
               "physicochemical", "alive", track->GetPosition());
  }
  for (int i = 0; i < 5; ++i) {
    SpatialEmpty("physical_sites", "excitation_" + std::to_string(i), "physical");
    SpatialEmpty("physical_sites", "ionisation_" + std::to_string(i), "physical");
    SpatialEmpty("pre_decay_water", "excitation_" + std::to_string(i), "physicochemical");
    SpatialEmpty("pre_decay_water", "ionisation_" + std::to_string(i), "physicochemical");
  }
  SpatialEmpty("physical_sites", "attachment", "physical");
  SpatialEmpty("physical_sites", "solvation", "physical");
  SpatialEmpty("pre_decay_water", "attachment", "physicochemical");
  SpatialEmpty("solvated_electron_creation", "e_aq", "physicochemical");
  for (const char* name : species) {
    SpatialEmpty("dissociation_products", name, "physicochemical");
    SpatialEmpty("final_survivors", name, "physicochemical");
  }
}

class SpatialSpeciesAction : public G4UserTimeStepAction {
 public:
  void StartProcessing() override { fInitialized = false; }
  void UserPreTimeStepAction() override
  {
    auto* scheduler = G4Scheduler::Instance();
    if (!fInitialized) {
      auto* tracks = G4ITTrackHolder::Instance()->GetMainList();
      if (!tracks || tracks->begin() == tracks->end())
        SpatialFailure("missing initial molecular track list");
      if (scheduler->GetTimeStep() != 0. ||
          std::abs(scheduler->GetGlobalTime() - ps) > 1e-12 * ps)
        SpatialFailure("initial molecules are not on the global 1 ps zero step");
      for (const G4Track* track : *tracks) SpatialInitialMolecule(track);
      fInitialized = true;
    }
    if (scheduler->GetTimeStep() > 0.)
      SpatialFailure("positive chemistry step selected before final snapshot");
  }
  void UserReactionAction(const G4Track&, const G4Track&, const std::vector<G4Track*>*) override
  { SpatialFailure("inter-molecular reaction preceded final snapshot"); }
  void UserPostTimeStepAction() override
  {
    auto* scheduler = G4Scheduler::Instance();
    auto* tracks = G4ITTrackHolder::Instance()->GetMainList();
    if (!tracks) SpatialFailure("missing post-step molecular track list");
    bool hasWater = false;
    std::map<long, unsigned long> processingByParent;
    for (const G4Track* track : *tracks) {
      if (track->GetDefinition() == G4H2O::Definition()) hasWater = true;
      if (gSpatialKnownMolecules.insert(track->GetTrackID()).second) {
        const long parent = track->GetParentID();
        auto inserted = processingByParent.emplace(parent, gSpatialProcessingOrdinal);
        if (inserted.second) ++gSpatialProcessingOrdinal;
        // Propagate lineage even for the unreported H2Ovib intermediate.
        const auto source = gSpatialMoleculeSource.find(parent);
        const auto physical = gSpatialMoleculePhysicalTrack.find(parent);
        if (source == gSpatialMoleculeSource.end() || physical == gSpatialMoleculePhysicalTrack.end())
          SpatialFailure("new molecule has no observed parent lineage");
        gSpatialMoleculeSource[track->GetTrackID()] = source->second;
        gSpatialMoleculePhysicalTrack[track->GetTrackID()] = physical->second;
        SpatialDaughter(track, inserted.first->second);
      }
    }
    if (hasWater) { ++gSpatialPass; return; }
    if (scheduler->GetTimeStep() != 0. ||
        std::abs(scheduler->GetGlobalTime() - ps) > 1e-12 * ps)
      SpatialFailure("final survivors are not on the global 1 ps zero step");
    SpatialCompleteHistory(tracks);
    gPreChemCaptured = true;
    scheduler->Stop();
  }
 private:
  bool fInitialized = false;
};

[[noreturn]] void PreChemFailure(const char* message)
{
  fprintf(stderr, "physchem_prechem: %s; output is incomplete, do not commit it\n", message);
  exit(1);
}

// Geant4 11.4.1: G4ITStepProcessor2.cc:110-185 merges decay daughters and removes
// killed parents before G4Scheduler.cc:700-701 calls the post action. While stopped
// water states remain, G4DNAIRTMoleculeEncounterStepper.cc:431-467 defers IRT
// initialization. Stop on the final water-decay zero step, before the next call to
// CalculateMinTimeStep initializes IRT (G4ITModelProcessor.cc:118-131).
class PreChemSpeciesAction : public G4UserTimeStepAction {
 public:
  void UserPreTimeStepAction() override
  {
    if (!gPreChemCaptured && G4Scheduler::Instance()->GetTimeStep() > 0.)
      PreChemFailure("positive time step selected before the water-decay snapshot");
  }
  void UserReactionAction(const G4Track&, const G4Track&,
                          const std::vector<G4Track*>*) override
  {
    PreChemFailure("an inter-molecular chemical reaction preceded the snapshot");
  }
  void UserPostTimeStepAction() override
  {
    if (gPreChemCaptured) return;
    auto* scheduler = G4Scheduler::Instance();
    auto* tracks = G4ITTrackHolder::Instance()->GetMainList();
    if (!tracks) PreChemFailure("missing main track list");
    for (const G4Track* track : *tracks) {
      if (track->GetDefinition() == G4H2O::Definition()) return;
    }
    if (scheduler->GetTimeStep() != 0. ||
        std::abs(scheduler->GetGlobalTime() - ps) > 1e-12 * ps)
      PreChemFailure("water decays did not complete at global 1 ps on a zero step");
    for (const G4Track* track : *tracks) {
      const auto* molecule = GetMolecule(track);
      if (!molecule || track->GetTrackStatus() != fAlive ||
          std::abs(track->GetGlobalTime() - ps) > 1e-12 * ps)
        PreChemFailure("snapshot contains a non-alive/non-molecular track or a non-1-ps birth");
      const std::string name = molecule->GetName();
      ++gPhysChemEventCount[name];
      gPhysChemEventR2[name] += track->GetPosition().mag2() / (nm * nm);
    }
    gPreChemCaptured = true;
    scheduler->Stop();
  }
};

// Initial species list at 1 ps. At the first stage the scheduler holds the undissociated states
// (H2O^0, H2O^1, e_aq); the dissociation (H2O^1 -> H3O^1 + OH, excitation channels) happens in
// zero-length steps at t = 1 ps. The first step with a non-zero time step starts after all of them:
// the main list then is the species list "at 1 ps, before diffusion or reactions". Snapshot it
// there and stop the scheduler (Q24).
class InitialSpeciesAction : public G4UserTimeStepAction {
 public:
  void StartProcessing() override { fDone = false; }
  void UserPreTimeStepAction() override
  {
    auto* scheduler = G4Scheduler::Instance();
    if (fDone || scheduler->GetTimeStep() <= 0.) return;
    fDone = true;
    gPhysChemEventCount.clear();
    gPhysChemEventR2.clear();
    G4TrackManyList* tracks = G4ITTrackHolder::Instance()->GetMainList();
    if (tracks != nullptr) {
      for (const G4Track* track : *tracks) {
        const auto* molecule = GetMolecule(track);
        if (molecule == nullptr || molecule->GetName() == "H2O") continue;
        const std::string name = molecule->GetName();
        ++gPhysChemEventCount[name];
        const double r = track->GetPosition().mag() / nm;
        gPhysChemEventR2[name] += r * r;
      }
    }
    scheduler->Stop();
  }

 private:
  bool fDone = false;
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
    if (IsChemistryMode()) fChemistry = new G4EmDNAChemistry_option3();
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
    if (gMode == Mode::TwoBodyHH) fGun->SetParticleDefinition(G4Geantino::Definition());
    else fGun->SetParticleDefinition(G4Electron::Definition());
    // along +z like the Rust runs (RunConfig default); the elastic model of Geant4 depends
    // (slightly) on the direction through CLHEP orthogonal(), see docs/09 Q13
    fGun->SetParticleMomentumDirection(G4ThreeVector(0., 0., 1.));
    fGun->SetParticlePosition(G4ThreeVector());
  }
  ~Gun() override { delete fGun; }
  void GeneratePrimaries(G4Event* ev) override
  {
    fGun->SetParticleEnergy(gMode == Mode::TwoBodyHH ? 1. * eV : gEnergy);
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
    if (gMode == Mode::SpatialDiagnostic) {
      const G4StepPoint* post = step->GetPostStepPoint();
      const G4VProcess* defining = post->GetProcessDefinedStep();
      const std::string process = defining ? defining->GetProcessName() : "";
      if (process == "e-_G4DNAExcitation" || process == "e-_G4DNAIonisation" ||
          process == "e-_G4DNAAttachment" || process == "e-_G4DNAElectronSolvation") {
        gSpatialSites.push_back({track->GetTrackID(), track->GetParentID(), post->GetPosition(),
                                 process, gSpatialSourceOrdinal++, false});
      }
      return;
    }
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
    if (IsChemistryMode() && !gChemistryRun && stackManager->GetNTotalTrack() == 0) {
      gChemistryRun = true;
      G4DNAChemistryManager::Instance()->Run();
    }
  }
  G4ClassificationOfNewTrack ClassifyNewTrack(const G4Track* t) override
  {
    if (gMode == Mode::Slow) return SlowClassify(t);
    if (t->GetParentID() == 0) return fUrgent;
    // physchem follows the whole cascade (as slow mode and the Rust runs): secondaries are
    // transported until solvation/attachment, and their ionisations/excitations seed the chemistry
    if (IsChemistryMode()) return fUrgent;
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
  void BeginOfEventAction(const G4Event* event) override
  {
    gL = gEdep = gRem = gNsec = gEsec = 0.;
    gEnd = G4ThreeVector();
    if (gMode == Mode::Slow) SlowBeginEvent();
    if (IsChemistryMode()) {
      gChemistryRun = false;
      gPreChemCaptured = false;
      gPhysChemEventCount.clear();
      gPhysChemEventR2.clear();
    }
    if (gMode == Mode::SpatialDiagnostic) {
      gSpatialHistory = event->GetEventID();
      gSpatialSourceOrdinal = gSpatialPass = gSpatialProcessingOrdinal = 0;
      gSpatialSites.clear();
      gSpatialKnownMolecules.clear();
      gSpatialMoleculeSource.clear();
      gSpatialMoleculePhysicalTrack.clear();
      gSpatialPopulated.clear();
    }
  }
  void EndOfEventAction(const G4Event*) override
  {
    if (gMode == Mode::Track) FinishEvent();  // mfp mode finishes inside the stepping action
    if (gMode == Mode::Slow) SlowFinishEvent();
    if (IsChemistryMode()) {
      if (gMode == Mode::PreChem && !gPreChemCaptured)
        PreChemFailure("event ended without a verified snapshot");
      if (gMode == Mode::SpatialDiagnostic && !gPreChemCaptured)
        SpatialFailure("event ended without a verified final snapshot");
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
    if (gMode == Mode::TwoBodyHH) {
      static TwoBodyGun twoBodyGun;
      G4DNAChemistryManager::Instance()->SetGun(&twoBodyGun);
    }
    SetUserAction(new Gun);
    SetUserAction(new Stepping);
    SetUserAction(new Stacking);
    SetUserAction(new EventAct);
    SetUserAction(new RunAct);
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
  fprintf(f, "# G4EmDNAChemistry_option3; alive species snapshot at 1 ps (after the zero-time dissociation steps), before diffusion/reactions\n");
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

void RunPreChem(const char* out, long seed, const std::vector<std::pair<double, long>>& cases)
{
  // Exclusive creation: a diagnostic must never replace historical reference evidence.
  FILE* f = fopen(out, "wx");
  if (!f) PreChemFailure("cannot create output (it may already exist)");
  WriteProvenance(f, "physchem_prechem", 0, seed);
  fprintf(f, "# G4EmDNAChemistry_option3; IRT selected; no reaction table or process changes\n");
  fprintf(f, "# snapshot_boundary: after_dissociation_before_irt; global_time_ps: 1; chemical_reactions: 0\n");
  fprintf(f, "energy_eV,n_events,species,sum_count,sum_count2,sum_r2_nm2\n");
  for (const auto& c : cases) {
    gEnergy = c.first * eV;
    gPhysChem.clear();
    G4RunManager::GetRunManager()->BeamOn((G4int)c.second);
    if (gPhysChem.n_events != c.second) PreChemFailure("incomplete event count");
    for (const auto& item : gPhysChem.sum_count) {
      fprintf(f, "%.17g,%ld,%s,%.17g,%.17g,%.17g\n", c.first, gPhysChem.n_events,
              item.first.c_str(), item.second, gPhysChem.sum_count2[item.first],
              gPhysChem.sum_r2_nm2[item.first]);
    }
    if (fflush(f) != 0) PreChemFailure("failed to flush output");
    fprintf(stderr, "physchem_prechem: %.17g eV done (%ld events)\n", c.first, gPhysChem.n_events);
  }
  if (fclose(f) != 0) PreChemFailure("failed to close output");
}

std::vector<long> ParseSeeds(const std::string& spec)
{
  std::vector<long> seeds;
  size_t start = 0;
  while (start < spec.size()) {
    size_t comma = spec.find(',', start);
    if (comma == std::string::npos) comma = spec.size();
    const std::string item = spec.substr(start, comma - start);
    char* end = nullptr;
    const long seed = strtol(item.c_str(), &end, 10);
    if (item.empty() || !end || *end != '\0' || seed <= 0) SpatialFailure("invalid seed list");
    seeds.push_back(seed);
    start = comma + 1;
  }
  if (seeds.empty()) SpatialFailure("empty seed list");
  return seeds;
}

void RunSpatialDiagnostic(const char* out, long histories, const std::vector<long>& seeds)
{
  if (histories <= 0 || histories > std::numeric_limits<G4int>::max())
    SpatialFailure("history count must be a positive G4int");
  gSpatialFile = fopen(out, "wx");
  if (!gSpatialFile) SpatialFailure("cannot create output (it may already exist)");
  fprintf(gSpatialFile, "# generator: g4rs_ref (validation/geant4/g4rs_ref), mode physchem_spatial_diagnostic\n");
  fprintf(gSpatialFile, "# geant4_version_number: %d (%s)\n", G4VERSION_NUMBER, G4Version.c_str());
  fprintf(gSpatialFile, "# physics: G4EmDNAPhysics_option2; chemistry: G4EmDNAChemistry_option3; material: G4_WATER; fluo/auger: OFF\n");
  fprintf(gSpatialFile, "# primary_energy_eV: 900000; primary_origin_nm: 0,0,0; primary_direction: 0,0,1\n");
  fprintf(gSpatialFile, "# global_time_ps: 1; chemical_reactions: 0; histories_per_seed: %ld; expected_seed_count: %zu\n", histories, seeds.size());
  fprintf(gSpatialFile, "# boundaries: physical_sites,solvated_electron_creation,pre_decay_water,dissociation_products,final_survivors\n");
  fprintf(gSpatialFile, "# completion: valid only after every requested BeamOn returns and this file closes successfully\n");
  fprintf(gSpatialFile, "%s\n", kSpatialHeader);
  gEnergy = 900000. * eV;
  for (long seed : seeds) {
    gSpatialSeed = seed;
    G4Random::setTheSeed(seed);
    G4RunManager::GetRunManager()->BeamOn((G4int)histories);
    if (fflush(gSpatialFile) != 0) SpatialFailure("failed to flush output");
    fprintf(stderr, "physchem_spatial_diagnostic: seed %ld done (%ld events)\n", seed, histories);
  }
  fprintf(gSpatialFile, "# complete: true; seeds: ");
  for (size_t i = 0; i < seeds.size(); ++i) fprintf(gSpatialFile, "%s%ld", i ? "," : "", seeds[i]);
  fprintf(gSpatialFile, "; histories_per_seed: %ld\n", histories);
  if (fclose(gSpatialFile) != 0) SpatialFailure("failed to close output");
  gSpatialFile = nullptr;
}

void RunTwoBodyHH(const char* out, long seed, long pairs)
{
  if (seed <= 0) TwoBodyFailure("seed must be positive");
  if (pairs <= 0 || pairs > std::numeric_limits<G4int>::max())
    TwoBodyFailure("pair count must be a positive G4int");
  FILE* f = fopen(out, "wx");
  if (!f) TwoBodyFailure("cannot create output (it may already exist)");
  fprintf(f, "# generator: g4rs_ref (validation/geant4/g4rs_ref), mode two_body_h_h\n");
  fprintf(f, "# geant4_version_number: %d (%s)\n", G4VERSION_NUMBER, G4Version.c_str());
  fprintf(f, "# chemistry: G4EmDNAChemistry_option3; timestep_model: SBS; reaction_id: 1\n");
  fprintf(f, "# reactants: H+H; initial_separation_nm: 0.2; seed: %ld; pairs: %ld\n",
          seed, pairs);
  fprintf(f, "# completion: valid only when complete=true and every row has total_pairs=pairs\n");
  fprintf(f, "tau,observation_time_ps,total_pairs,reacted_count,surviving_count\n");

  gTwoBodyCompleted = 0;
  std::fill(gTwoBodyReacted, gTwoBodyReacted + kTwoBodyPoints, 0L);
  std::fill(gTwoBodySurviving, gTwoBodySurviving + kTwoBodyPoints, 0L);
  G4Random::setTheSeed(seed);
  G4RunManager::GetRunManager()->BeamOn((G4int)pairs);
  if (gTwoBodyCompleted != pairs) TwoBodyFailure("incomplete chemistry history count");
  for (size_t i = 0; i < kTwoBodyPoints; ++i) {
    if (gTwoBodyReacted[i] + gTwoBodySurviving[i] != pairs)
      TwoBodyFailure("incomplete observation coverage");
    fprintf(f, "%.17g,%.17g,%ld,%ld,%ld\n", kTwoBodyTau[i],
            kTwoBodyTau[i] * kTwoBodyTimeScalePs, pairs,
            gTwoBodyReacted[i], gTwoBodySurviving[i]);
  }
  fprintf(f, "# complete: true; seed: %ld; pairs: %ld; rows: %zu\n",
          seed, pairs, kTwoBodyPoints);
  if (fclose(f) != 0) TwoBodyFailure("failed to close output");
}


// R04 uses the public isolated-pair kernel, not a finite-time scheduler proxy.
// Source: G4DNAIRT::GetIndependentReactionTime (Geant4 11.4.1).
[[noreturn]] void PairMatrixFailure(const char* message)
{
  fprintf(stderr, "pair_matrix_irt: %s; output incomplete, do not commit\n", message);
  exit(1);
}

unsigned long long ParsePairInteger(const char* text)
{
  if (!text || !*text) PairMatrixFailure("empty integer");
  for (const char* p = text; *p; ++p)
    if (*p < '0' || *p > '9') PairMatrixFailure("expected unsigned decimal integer");
  errno = 0;
  char* end = nullptr;
  const auto value = strtoull(text, &end, 10);
  if (errno == ERANGE || !end || *end || !value) PairMatrixFailure("integer out of range");
  return value;
}

int RunPairMatrix(int argc, char** argv)
{
  if (argc != 5) PairMatrixFailure("expected output path, seed and pairs per case/distance");
  const auto seed = ParsePairInteger(argv[3]);
  const auto pairs = ParsePairInteger(argv[4]);
  // Injective base decomposition into the two valid Ranecu seed ranges, also on Windows.
  constexpr unsigned long long seedBase = 2147483562ULL;
  if (seed >= seedBase * 2147483398ULL || pairs > 1000000ULL)
    PairMatrixFailure("seed exceeds Ranecu encoding or pair count exceeds preregistered cap");
  if (G4VERSION_NUMBER != 1141) PairMatrixFailure("requires Geant4 11.4.1");
  FILE* f = fopen(argv[2], "wx");
  if (!f) PairMatrixFailure("cannot exclusively create output");
  gMode = Mode::PhysChem; // only selects the existing option3 physics constructor
  auto* param = G4EmParameters::Instance();
  param->SetTimeStepModel(G4ChemTimeStepModel::IRT);
  param->SetFluo(false);
  param->SetAuger(false);
  param->SetAugerCascade(false);
  param->SetPixe(false);
  auto* rm = G4RunManagerFactory::CreateRunManager(G4RunManagerType::Serial);
  rm->SetUserInitialization(new Detector);
  rm->SetUserInitialization(new Physics);
  rm->Initialize();
  G4LossTableManager::Instance()->SetAtomDeexcitation(nullptr);
  G4DNAChemistryManager::Instance()->Initialize();
  auto* table = G4DNAMolecularReactionTable::GetReactionTable();
  G4DNAIRT sampler;
  sampler.SetReactionTable(table);
  auto* oldEngine = G4Random::getTheEngine();
  CLHEP::RanecuEngine engine;
  const long seeds[2] = {static_cast<long>(seed % seedBase + 1),
                         static_cast<long>(seed / seedBase + 1)};
  engine.setSeeds(seeds, 0);
  G4Random::setTheEngine(&engine);
  fprintf(f, "# generator: g4rs_ref; mode: pair_matrix_irt; schema: r04_pair_v1\n");
  fprintf(f, "# geant4_version_number: %d (%s); chemistry: G4EmDNAChemistry_option3; timestep_model: IRT\n", G4VERSION_NUMBER, G4Version.c_str());
  fprintf(f, "# seed: %llu; rng: RanecuEngine; engine_seeds: %ld,%ld; pairs: %llu\n", seed, seeds[0], seeds[1], pairs);
  fprintf(f, "# direct G4DNAIRT pair kernel; negative -1 ps sentinel means eventual nonreaction; no scheduler cutoff\n");
  fprintf(f, "reaction_id,r0_over_a,a_nm,D_nm2_per_ps,onsager_nm,kobs_M_inv_s_inv,endpoint,tau,observation_time_ps,total_pairs,event_count\n");
  const int ids[] = {1, 2, 8, 9, 11, 17, 42, 51};
  const double distances[] = {1., 1.5, 3.};
  const double taus[] = {0.01, 0.1, 1., 10., 100.};
  size_t rows = 0;
  for (int id : ids) {
    const auto* reaction = table->GetReaction(id);
    if (!reaction || reaction->GetReactionType() != (id >= 11 ? 1 : 0))
      PairMatrixFailure("missing channel or wrong IRT type");
    const auto* a = reaction->GetReactant1();
    const auto* b = reaction->GetReactant2();
    const double radius = id >= 11 ? reaction->GetReactionRadius() : reaction->GetEffectiveReactionRadius();
    const double diffusion = a->GetDiffusionCoefficient() + b->GetDiffusionCoefficient();
    if (!(radius > 0 && diffusion > 0) || !std::isfinite(radius + diffusion))
      PairMatrixFailure("invalid option3 parameters");
    fprintf(f, "# channel: %d; reactants: %s,%s; effective_radius_nm: %.17g; contact_probability: %.17g\n", id,
            a->GetName().c_str(), b->GetName().c_str(), reaction->GetEffectiveReactionRadius()/nm, reaction->GetProbability());
    for (double distance : distances) {
      unsigned long long counts[6] = {};
      const double scale = radius * radius / diffusion;
      for (unsigned long long pair = 0; pair < pairs; ++pair) {
        const double time = sampler.GetIndependentReactionTime(a, b, distance * radius);
        if (!std::isfinite(time) || (time < 0 && time != -1 * ps))
          PairMatrixFailure("unexpected IRT result");
        if (time < 0) ++counts[5];
        else for (size_t i = 0; i < 5; ++i)
          if (time <= taus[i] * scale) ++counts[i];
      }
      for (size_t i = 0; i < 6; ++i) {
        fprintf(f, "%d,%.17g,%.17g,%.17g,%.17g,%.17g,%s,", id, distance,
                radius/nm, diffusion/(nm*nm/ps), reaction->GetOnsagerRadius()/nm,
                reaction->GetObservedReactionRateConstant()/(1e-3*m3/(mole*s)),
                i == 5 ? "no_reaction" : "cdf");
        if (i == 5) fprintf(f, "inf,inf,");
        else fprintf(f, "%.17g,%.17g,", taus[i], taus[i]*scale/ps);
        fprintf(f, "%llu,%llu\n", pairs, counts[i]);
        ++rows;
      }
      if (fflush(f) || ferror(f)) PairMatrixFailure("output write failed");
    }
    fprintf(stderr, "pair_matrix_irt: channel %d complete\n", id);
  }
  if (rows != 144) PairMatrixFailure("incomplete matrix");
  fprintf(f, "# complete: true; seed: %llu; pairs: %llu; rows: %zu\n", seed, pairs, rows);
  if (fclose(f)) PairMatrixFailure("output close failed");
  G4Random::setTheEngine(oldEngine);
  delete rm;
  return 0;
}

}  // namespace

int main(int argc, char** argv)
{
  if (argc > 1 && std::string(argv[1]) == "pair_matrix_irt") return RunPairMatrix(argc, argv);
  if (argc < 3) {
    fprintf(stderr, "usage: %s xs <out.csv> | mfp <out.csv> <events> <seed> | track <out.csv> <events> <seed> <cut_eV> | slow <out_prefix> <seed> [E_eV:events,...] | physchem <out.csv> <seed> [E_eV:events,...] | physchem_prechem <out.csv> <seed> <E_eV:events,...> | physchem_spatial_diagnostic <out.csv> <histories_per_seed> <seed,...> | two_body_h_h <out.csv> <seed> <pairs>\n", argv[0]);
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
  else if (mode == "physchem_prechem") {
    if (argc != 5) PreChemFailure("expected output path, seed and energy:events list");
    seed = atol(argv[3]);
    slowSpec = argv[4];
    const auto cases = ParseSpec(slowSpec);
    if (seed <= 0 || cases.empty()) PreChemFailure("seed/statistics must be positive");
    for (const auto& c : cases) {
      if (!std::isfinite(c.first) || c.first < 2. || c.first >= 1e6 || c.second <= 0 ||
          c.second > std::numeric_limits<G4int>::max())
        PreChemFailure("requires 2 <= energy_eV < 1e6 and positive G4int event counts");
    }
    gMode = Mode::PreChem;
  }
  else if (mode == "physchem_spatial_diagnostic") {
    if (argc != 5) SpatialFailure("expected output path, histories per seed and seed list");
    events = atol(argv[3]);
    ParseSeeds(argv[4]);  // validate before Geant4 initialization
    if (events <= 0) SpatialFailure("histories per seed must be positive");
    gMode = Mode::SpatialDiagnostic;
  }
  else if (mode == "two_body_h_h") {
    if (argc != 5) TwoBodyFailure("expected output path, seed and pair count");
    char* seedEnd = nullptr;
    char* pairEnd = nullptr;
    errno = 0;
    seed = strtol(argv[3], &seedEnd, 10);
    const bool seedOutOfRange = errno == ERANGE;
    errno = 0;
    events = strtol(argv[4], &pairEnd, 10);
    const bool pairsOutOfRange = errno == ERANGE;
    if (!seedEnd || *seedEnd != '\0' || seedOutOfRange || seed <= 0)
      TwoBodyFailure("seed must be a positive integer");
    if (!pairEnd || *pairEnd != '\0' || pairsOutOfRange || events <= 0 ||
        events > std::numeric_limits<G4int>::max())
      TwoBodyFailure("pair count must be a positive G4int");
    gMode = Mode::TwoBodyHH;
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
  // chem6 sets "/process/chem/TimeStepModel IRT" before /run/initialize. The default is Unknown:
  // no step model is registered and G4Scheduler::Process crashes in G4ITModelManager::Initialize
  // (Q24). It must be set before the physics list is constructed (ConstructProcess reads it).
  if (IsChemistryMode())
    param->SetTimeStepModel(gMode == Mode::TwoBodyHH ? G4ChemTimeStepModel::SBS
                                                    : G4ChemTimeStepModel::IRT);

  G4Random::setTheSeed(seed);
  rm->Initialize();
  // EndTime must exceed the 1 ps species birth time for StartProcessing to be entered; the user
  // action records the initial list and calls Stop() before the first chemistry step.
  if (IsChemistryMode()) {
    G4Scheduler::Instance()->SetEndTime(gMode == Mode::TwoBodyHH
                                            ? kTwoBodyTau[kTwoBodyPoints - 1] *
                                                  kTwoBodyTimeScalePs * ps
                                            : 2. * ps);
    if (gMode == Mode::PreChem) G4Scheduler::Instance()->SetUserAction(new PreChemSpeciesAction());
    else if (gMode == Mode::SpatialDiagnostic)
      G4Scheduler::Instance()->SetUserAction(new SpatialSpeciesAction());
    else if (gMode == Mode::TwoBodyHH) {
      G4Scheduler::Instance()->SetUserAction(new TwoBodyAction());
      for (G4double tau : kTwoBodyTau)
        G4Scheduler::Instance()->AddWatchedTime(tau * kTwoBodyTimeScalePs * ps);
    }
    else G4Scheduler::Instance()->SetUserAction(new InitialSpeciesAction());
  }

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
  else if (gMode == Mode::PreChem) {
    RunPreChem(out, seed, ParseSpec(slowSpec));
  }
  else if (gMode == Mode::SpatialDiagnostic) {
    RunSpatialDiagnostic(out, events, ParseSeeds(argv[4]));
  }
  else if (gMode == Mode::TwoBodyHH) {
    RunTwoBodyHH(out, seed, events);
  }
  else {
    RunEvents(out, "track", {200., 1e3, 1e4, 1e5, 9e5}, events, seed);
  }
  delete rm;
  return 0;
}
