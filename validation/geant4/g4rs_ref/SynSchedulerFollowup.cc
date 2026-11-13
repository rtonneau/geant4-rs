// Ordinary installed-library IRT-syn trace: no renamed source or draw interception.
#include "G4DNAChemistryManager.hh"
#include "G4DNAMolecularReactionTable.hh"
#include "G4EmParameters.hh"
#include "G4EmDNAPhysics_option2.hh"
#include "G4EmDNAChemistry_option3.hh"
#include "G4RunManagerFactory.hh"
#include "G4VModularPhysicsList.hh"
#include "G4VUserDetectorConstruction.hh"
#include "G4NistManager.hh"
#include "G4Sphere.hh"
#include "G4LogicalVolume.hh"
#include "G4PVPlacement.hh"
#include "G4LossTableManager.hh"
#include "G4UserTimeStepAction.hh"
#include "G4DNAIndependentReactionTimeModel.hh"
#include "G4DNAIndependentReactionTimeStepper.hh"
#include "G4DNAMakeReaction.hh"
#include "G4ITTrackHolder.hh"
#include "G4Scheduler.hh"
#include "G4Molecule.hh"
#include "G4SystemOfUnits.hh"
#include "G4PhysicalConstants.hh"
#include "G4Version.hh"
#include "Randomize.hh"
#include "CLHEP/Random/RanecuEngine.h"
#include <array>
#include <cerrno>
#include <cstdio>
#include <cstdlib>
#include <stdexcept>
#include <fstream>
#include <vector>
class TraceDetector:public G4VUserDetectorConstruction {
 public:G4VPhysicalVolume* Construct() override {
  auto* water=G4NistManager::Instance()->FindOrBuildMaterial("G4_WATER");
  auto* world=new G4LogicalVolume(new G4Sphere("World",0.,1.*m,0.,twopi,0.,pi),water,"World");
  return new G4PVPlacement(nullptr,{},world,"World",nullptr,false,0);
 }
};
class TracePhysics:public G4VModularPhysicsList {
 G4EmDNAPhysics_option2* physics=new G4EmDNAPhysics_option2;
 G4EmDNAChemistry_option3* chemistry=new G4EmDNAChemistry_option3;
 public:
 ~TracePhysics() override {delete physics;delete chemistry;}
 void ConstructParticle() override {physics->ConstructParticle();chemistry->ConstructParticle();}
 void ConstructProcess() override {if(G4EmParameters::Instance()->GetTimeStepModel()!=G4ChemTimeStepModel::IRT_syn)throw std::runtime_error("IRT_syn missing before ConstructProcess");AddTransportation();physics->ConstructProcess();chemistry->ConstructProcess();}
};
class TraceAction:public G4UserTimeStepAction {
 public:
 std::array<const G4Track*,2> parents{};
 std::array<int,2> ids{};
 const G4DNAMolecularReactionData* reaction=nullptr;
 bool reacted=false,removed=false,modelVerified=false;
 double time=0.,step=0.;
 std::array<G4ThreeVector,2> positions{};
 G4ThreeVector site;
 int products=0;
 void StartProcessing() override {
  // Called by the installed scheduler after its model processor initializes.
  auto models=G4Scheduler::Instance()->GetModelHandler()->GetActiveModels(ps);
  if(models.size()!=1 || !dynamic_cast<G4DNAIndependentReactionTimeModel*>(models[0])
     || !dynamic_cast<G4DNAIndependentReactionTimeStepper*>(models[0]->GetTimeStepper())
     || !dynamic_cast<G4DNAMakeReaction*>(models[0]->GetReactionProcess()))
   throw std::runtime_error("ordinary IRT_syn model/stepper/reaction process not registered");
  modelVerified=true;
 }
 void UserReactionAction(const G4Track& a,const G4Track& b,const std::vector<G4Track*>* out) override {
  if(reacted || !((&a==parents[0]&&&b==parents[1])||(&a==parents[1]&&&b==parents[0])))throw std::runtime_error("unexpected reacting parents");
  reacted=true;positions={parents[0]->GetPosition(),parents[1]->GetPosition()};
  time=G4Scheduler::Instance()->GetGlobalTime()/ps;step=G4Scheduler::Instance()->GetTimeStep()/ps;
  products=out?static_cast<int>(out->size()):0;
  for(int i=0;i<products;++i){
   if(i==0)site=(*out)[i]->GetPosition();
  }
  G4Scheduler::Instance()->Stop();
 }
 void UserPostTimeStepAction() override {
  if(!reacted)return;
  removed=true;
  auto* list=G4ITTrackHolder::Instance()->GetMainList();
  if(list)for(const auto* track:*list)if(track->GetTrackID()==ids[0]||track->GetTrackID()==ids[1])removed=false;
 }
};
unsigned long long Integer(const char* text){
 if(!text||!*text)throw std::runtime_error("empty integer");
 for(auto* c=text;*c;++c)if(*c<'0'||*c>'9')throw std::runtime_error("invalid integer");
 errno=0;char* end=nullptr;auto value=strtoull(text,&end,10);
 if(errno||!value||!end||*end)throw std::runtime_error("integer out of range");return value;
}
void Vector(FILE* f,const G4ThreeVector& p){for(double value:{p.x()/nm,p.y()/nm,p.z()/nm}){if(!std::isfinite(value))throw std::runtime_error("nonfinite coordinate");fprintf(f,",%.17g",value);}}
int main(int argc,char** argv){try{
 if(argc!=6||std::string(argv[1])!="followup")throw std::runtime_error("expected followup OUTPUT MANIFEST REPLICA HISTORIES");
 if(G4VERSION_NUMBER!=1141)throw std::runtime_error("requires normative Geant4 11.4.1");
 auto replica=Integer(argv[4]),trials=Integer(argv[5]);
 if((replica!=1 && replica!=2)||(trials!=1 && trials!=2048))throw std::runtime_error("invalid replica or histories");
 std::ifstream manifest(argv[3]);
 std::vector<std::array<long,2>> allSeeds;
 long firstSeed,secondSeed;
 while(manifest >> firstSeed >> secondSeed) {
  if(firstSeed<1||firstSeed>2147483562L||secondSeed<1||secondSeed>2147483398L)throw std::runtime_error("invalid manifest seed");
  allSeeds.push_back({firstSeed,secondSeed});
 }
 if(!manifest.eof()||allSeeds.size()!=6144)throw std::runtime_error("invalid manifest");
 FILE* file=fopen(argv[2],"wx");if(!file)throw std::runtime_error("output must be new");
  auto* manager=G4RunManagerFactory::CreateRunManager(G4RunManagerType::Serial);
 manager->SetUserInitialization(new TraceDetector);manager->SetUserInitialization(new TracePhysics);
 // G4EmDNAPhysics construction resets EmParameters (including the model).
 // Configure only after construction, before ConstructProcess/chemistry setup.
 auto* param=G4EmParameters::Instance();param->SetTimeStepModel(G4ChemTimeStepModel::IRT_syn);
 param->SetFluo(false);param->SetAuger(false);param->SetAugerCascade(false);param->SetPixe(false);
 if(param->GetTimeStepModel()!=G4ChemTimeStepModel::IRT_syn)throw std::runtime_error("IRT_syn configuration failed");
 manager->Initialize();
 if(param->GetTimeStepModel()!=G4ChemTimeStepModel::IRT_syn)throw std::runtime_error("IRT_syn reset during initialization");
 fprintf(stderr,"syn scheduler: verified IRT_syn after physics construction and initialization\n");
 G4LossTableManager::Instance()->SetAtomDeexcitation(nullptr);
 // No transport BeamOn is performed: use the chemistry manager's supported
 // standalone setup to build molecular process tables and close geometry.
 G4DNAChemistryManager::Instance()->UseAsStandalone(true);
 G4DNAChemistryManager::Instance()->Initialize();
 auto* scheduler=G4Scheduler::Instance();scheduler->SetEndTime(1000000*ps);
 auto* table=G4DNAMolecularReactionTable::GetReactionTable();
 auto* oldEngine=G4Random::getTheEngine();CLHEP::RanecuEngine engine;G4Random::setTheEngine(&engine);
 fprintf(file,"# schema: syn_scheduler_followup_v1; Geant4: 1141\n");
 fprintf(file,"# replica: %llu; histories: %llu; birth_ps: 1; horizon_ps: 1000000\n",replica,trials);
 fprintf(file,"seed1,seed2,channel,orientation,trial,outcome,a_nm,d1_nm2_ps,d2_nm2_ps,p1x,p1y,p1z,p2x,p2y,p2z,event_time_ps,event_step_ps,r1x,r1y,r1z,r2x,r2y,r2z,products,px,py,pz,parents_removed\n");
 size_t rows=0;
 for(int id:{11})for(int orientation:{0,1,2})for(unsigned trial=0;trial<trials;++trial){
  const auto& pair=allSeeds[orientation*2048+trial];
  const long seeds[2]={pair[0],pair[1]};engine.setSeeds(seeds,0);
  auto* reaction=table->GetReaction(id);double a=reaction->GetEffectiveReactionRadius();
  G4ThreeVector origin=orientation==2?G4ThreeVector(10*a,-7*a,3*a):G4ThreeVector();
  G4ThreeVector delta=orientation==1?G4ThreeVector(0,0,3*a):G4ThreeVector(3*a,0,0);
  TraceAction action;action.reaction=reaction;
  auto* first=(new G4Molecule(reaction->GetReactant1()))->BuildTrack(ps,origin);
  auto* second=(new G4Molecule(reaction->GetReactant2()))->BuildTrack(ps,origin+delta);
  G4ITTrackHolder::Instance()->Push(first);G4ITTrackHolder::Instance()->Push(second);
  action.parents={first,second};action.ids={first->GetTrackID(),second->GetTrackID()};scheduler->SetUserAction(&action);
  G4DNAChemistryManager::Instance()->Run();scheduler->SetUserAction(nullptr);
  if(!action.modelVerified)throw std::runtime_error("ordinary scheduler did not verify its initialized model");
  fprintf(file,"%ld,%ld,%d,%d,%u,%s,%.17g,%.17g,%.17g",pair[0],pair[1],id,orientation,trial,action.reacted?"reaction":"censored",a/nm,reaction->GetReactant1()->GetDiffusionCoefficient()/(nm*nm/ps),reaction->GetReactant2()->GetDiffusionCoefficient()/(nm*nm/ps));Vector(file,origin);Vector(file,origin+delta);
  if(action.reacted){
   fprintf(file,",%.17g,%.17g",action.time,action.step);Vector(file,action.positions[0]);Vector(file,action.positions[1]);fprintf(file,",%d",action.products);
   if(action.products)Vector(file,action.site);else fprintf(file,",NA,NA,NA");fprintf(file,",%d\n",action.removed?1:0);
  }else fprintf(file,",NA,NA,NA,NA,NA,NA,NA,NA,NA,NA,NA,NA,NA\n");
  if(fflush(file)||ferror(file))throw std::runtime_error("CSV write failed");++rows;
 }
 if(rows!=3*trials||ftell(file)>10485000)throw std::runtime_error("incomplete/oversized fixture");
 fprintf(file,"# complete: true; replica: %llu; histories: %llu; rows: %zu\n",replica,trials,rows);
 if(fclose(file))throw std::runtime_error("CSV close failed");G4Random::setTheEngine(oldEngine);delete manager;return 0;
}catch(const std::exception& e){fprintf(stderr,"syn scheduler: %s; incomplete output is not evidence\n",e.what());return 1;}}
