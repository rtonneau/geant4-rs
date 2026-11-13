// New, independent IRT-syn diagnostics. Existing generator modes are unchanged.
// Included after the shared initialization helpers in g4rs_ref.cc.
#include "G4DiffusionControlledReactionModel.hh"

int RunSynPairMatrix(int argc, char** argv)
{
  if (argc != 5) PairMatrixFailure("syn_pair_matrix requires output, seed, samples");
  const auto seed = ParsePairInteger(argv[3]);
  const auto samples = ParsePairInteger(argv[4]);
  constexpr unsigned long long seedBase = 2147483562ULL;
  if (seed >= seedBase * 2147483398ULL || samples > 1000000ULL)
    PairMatrixFailure("syn seed/statistics exceeds limit");
  if (G4VERSION_NUMBER != 1141) PairMatrixFailure("syn requires normative 11.4.1");
  FILE* output = fopen(argv[2], "wx");
  if (!output) PairMatrixFailure("syn output must be a new file");
  gMode = Mode::PhysChem;
  auto* parameters = G4EmParameters::Instance();
  parameters->SetTimeStepModel(G4ChemTimeStepModel::IRT_syn);
  parameters->SetFluo(false);
  parameters->SetAuger(false);
  parameters->SetAugerCascade(false);
  parameters->SetPixe(false);
  auto* manager = G4RunManagerFactory::CreateRunManager(G4RunManagerType::Serial);
  manager->SetUserInitialization(new Detector);
  manager->SetUserInitialization(new Physics);
  manager->Initialize();
  G4LossTableManager::Instance()->SetAtomDeexcitation(nullptr);
  G4DNAChemistryManager::Instance()->Initialize();
  auto* table = G4DNAMolecularReactionTable::GetReactionTable();
  G4DiffusionControlledReactionModel model;
  model.SetReactionTable(table);
  auto* previousEngine = G4Random::getTheEngine();
  CLHEP::RanecuEngine engine;
  const long seeds[2] = {static_cast<long>(seed % seedBase + 1), static_cast<long>(seed / seedBase + 1)};
  engine.setSeeds(seeds, 0);
  G4Random::setTheEngine(&engine);
  fprintf(output, "# schema: syn_pair_v1; Geant4: %d; model: IRT_syn; constructor: G4EmDNAChemistry_option3\n", G4VERSION_NUMBER);
  fprintf(output, "# seed: %llu; samples: %llu; overlap: source branch diagnostic, inherited candidate deliberately unresolved\n", seed, samples);
  fprintf(output, "reaction_id,context,r_over_a,a_nm,D_nm2_per_ps,contact_probability,endpoint,tau,samples,count\n");
  size_t rows = 0;
  for (int id : {1, 2, 8, 9, 11, 17, 42, 51}) {
    const auto* reaction = table->GetReaction(id);
    if (!reaction || reaction->GetReactionType() != (id >= 11 ? 1 : 0)) PairMatrixFailure("syn missing reaction");
    const double radius = reaction->GetEffectiveReactionRadius();
    const double diffusion = reaction->GetReactant1()->GetDiffusionCoefficient() + reaction->GetReactant2()->GetDiffusionCoefficient();
    const double scale = radius * radius / diffusion;
    if (!(radius > 0 && diffusion > 0) || !std::isfinite(scale)) PairMatrixFailure("syn bad source parameters");
    for (double ratio : {0.5, 1., 1.5, 3.}) {
      std::unique_ptr<G4Track> first((new G4Molecule(reaction->GetReactant1()))->BuildTrack(0., G4ThreeVector()));
      std::unique_ptr<G4Track> second((new G4Molecule(reaction->GetReactant2()))->BuildTrack(0., G4ThreeVector(ratio * radius, 0., 0.)));
      unsigned long long counts[7] = {};
      const double taus[] = {0.01, 0.1, 1., 10., 100.};
      for (unsigned long long sample = 0; sample < samples; ++sample) {
        double time = 0;
        if (ratio < 1.) {
          // Stepper CalculateStep's strict initial-overlap probability branch.
          // Rejection leaves prior fSampledMinTimeStep untouched, not a -1 ps
          // encounter sentinel. Keep this as a diagnostic outcome of its own.
          if (!(reaction->GetProbability() > G4UniformRand())) { ++counts[6]; continue; }
        } else time = model.GetTimeToEncounter(*first, *second);
        if (!std::isfinite(time) || (time < 0 && time != -1 * ps)) PairMatrixFailure("syn unexpected encounter result");
        if (time < 0) ++counts[5];
        else for (size_t endpoint = 0; endpoint < 5; ++endpoint)
          if (time <= taus[endpoint] * scale) ++counts[endpoint];
      }
      for (size_t endpoint = 0; endpoint < 7; ++endpoint) {
        fprintf(output, "%d,initial,%.17g,%.17g,%.17g,%.17g,%s,%.17g,%llu,%llu\n", id, ratio, radius/nm, diffusion/(nm*nm/ps), reaction->GetProbability(), endpoint < 5 ? "cdf" : endpoint == 5 ? "no_reaction" : "unresolved_overlap", endpoint < 5 ? taus[endpoint] : -1., samples, counts[endpoint]);
        ++rows;
      }
    }
    fprintf(output, "%d,post_start,0.5,%.17g,%.17g,%.17g,unresolved_overlap,-1,%llu,%llu\n", id, radius/nm, diffusion/(nm*nm/ps), reaction->GetProbability(), samples, samples);
    ++rows;
    if (fflush(output) || ferror(output)) PairMatrixFailure("syn output write failed");
    fprintf(stderr, "syn_pair_matrix: channel %d complete\n", id);
  }
  if (rows != 232) PairMatrixFailure("syn incomplete matrix");
  fprintf(output, "# complete: true; seed: %llu; samples: %llu; rows: %zu\n", seed, samples, rows);
  if (fclose(output)) PairMatrixFailure("syn output close failed");
  G4Random::setTheEngine(previousEngine);
  delete manager;
  return 0;
}
