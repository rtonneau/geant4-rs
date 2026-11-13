// Mapping diagnostic only. CMake pins the exact unmodified source file below.
#define G4DNAMakeReaction G4rsSynMakeReaction
#include "G4DNAMakeReaction.hh"
#undef G4DNAMakeReaction
#include "G4DNAMolecularReactionTable.hh"
#include "G4VDNAReactionModel.hh"
#include "G4Molecule.hh"
#include "G4MoleculeFinder.hh"
#include "G4ITReactionChange.hh"
#include "Randomize.hh"
#include "G4SystemOfUnits.hh"
#include "G4ITReaction.hh"
#include "G4DNAIndependentReactionTimeStepper.hh"
#include "G4Scheduler.hh"
#include "G4UnitsTable.hh"
#include "G4DNAScavengerMaterial.hh"
#include "G4MoleculeCounterManager.hh"
#include <stdexcept>
#include <array>
#include <cstdio>
#include <cstdlib>
#include <cerrno>
#include <memory>

struct SynInputs {double zx,zy,zz,phi,theta,product; unsigned mask=0;};
static SynInputs synInputs;
void SynConsume(unsigned bit) {
  if (synInputs.mask & bit) throw std::runtime_error("repeated semantic draw");
  synInputs.mask |= bit;
}
template<int Line> struct SynNormal {
  static double shoot(double mean,double width) {
    unsigned bit; double value;
    if constexpr(Line==188) {bit=1;value=synInputs.zx;}
    else if constexpr(Line==189) {bit=2;value=synInputs.zy;}
    else if constexpr(Line==190) {bit=4;value=synInputs.zz;}
    else {throw std::runtime_error("unknown source normal label");}
    SynConsume(bit); return mean+width*value;
  }
};
double SynUniform(int line) {
  if(line==192) {SynConsume(8);return synInputs.phi;}
  if(line==195) {SynConsume(16);return synInputs.theta;}
  if(line==107) {SynConsume(32);return synInputs.product;}
  throw std::runtime_error("unknown source uniform label");
}
// Headers above are loaded before interception: only this source body's draws
// are replaced. Semantic source lines make argument evaluation order irrelevant.
#undef G4RandGauss
#undef G4UniformRand
#define G4DNAMakeReaction G4rsSynMakeReaction
#define G4RandGauss SynNormal<__LINE__>
#define G4UniformRand() SynUniform(__LINE__)
#include G4RS_SYN_MAKE_SOURCE
#undef G4DNAMakeReaction
#undef G4RandGauss
#undef G4UniformRand
#define G4RandGauss CLHEP::RandGaussQ
#define G4UniformRand() CLHEP::HepRandom::getTheEngine()->flat()

#include "G4DiffusionControlledReactionModel.hh"
#include "G4DNAChemistryManager.hh"
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
#include "G4VExceptionHandler.hh"
#include "G4StateManager.hh"
#include "G4Version.hh"
#include "G4PhysicalConstants.hh"

class SynDetector:public G4VUserDetectorConstruction {
 public:G4VPhysicalVolume* Construct() override {
    auto* water=G4NistManager::Instance()->FindOrBuildMaterial("G4_WATER");
    auto* solid=new G4Sphere("World",0.,1.*m,0.,twopi,0.,pi);
    auto* logical=new G4LogicalVolume(solid,water,"World");
    return new G4PVPlacement(nullptr,{},logical,"World",nullptr,false,0);
  }
};
class SynPhysics:public G4VModularPhysicsList {
  G4EmDNAPhysics_option2 physics;
  G4EmDNAChemistry_option3 chemistry;
 public:
  void ConstructParticle() override {physics.ConstructParticle();chemistry.ConstructParticle();}
  void ConstructProcess() override {AddTransportation();physics.ConstructProcess();chemistry.ConstructProcess();}
};
class SynCoincidence:public std::exception {};
class SynExceptions:public G4VExceptionHandler {
 public:G4bool Notify(const char*,const char* code,G4ExceptionSeverity severity,const char* message) override {
    if(std::string(code)=="G4DNAMakeReaction003" && severity==FatalErrorInArgument) throw SynCoincidence();
    throw std::runtime_error(std::string(code)+": "+message);
  }
};
unsigned long long SynInteger(const char* text) {
  if(!text || !*text) throw std::runtime_error("empty integer");
  for(auto p=text;*p;++p) if(*p<'0'||*p>'9') throw std::runtime_error("invalid integer");
  errno=0;char* end=nullptr;auto value=strtoull(text,&end,10);
  if(errno || !value || !end || *end) throw std::runtime_error("integer range");return value;
}
void SynVector(FILE* file,const G4ThreeVector& p) {
  for(double x:{p.x()/nm,p.y()/nm,p.z()/nm}) {
    if(!std::isfinite(x)) throw std::runtime_error("nonfinite source coordinate");
    fprintf(file,",%.17g",x);
  }
}
void SynTape(unsigned long long seed,unsigned tape) {
  int j=static_cast<int>((tape+seed%16)%16);
  synInputs={ (j%7-3)*.75, ((3*j+1)%7-3)*.5, ((5*j+2)%7-3)*1.25,
    (j%8+.5)/8.,((3*j+1)%8+.5)/8.,((5*j+2)%8+.5)/8.,0};
}
int main(int argc,char** argv) {
 try {
  if(argc!=5 || std::string(argv[1])!="paired") throw std::runtime_error("expected paired OUTPUT SEED TAPES");
  if(G4VERSION_NUMBER!=1141) throw std::runtime_error("requires normative Geant4 11.4.1");
  auto seed=SynInteger(argv[3]);auto tapes=SynInteger(argv[4]);
  if(tapes>16) throw std::runtime_error("at most 16 registered primitive tapes");
  FILE* file=fopen(argv[2],"wx");if(!file) throw std::runtime_error("output must be new");
  auto* parameters=G4EmParameters::Instance();parameters->SetTimeStepModel(G4ChemTimeStepModel::IRT_syn);
  parameters->SetFluo(false);parameters->SetAuger(false);parameters->SetAugerCascade(false);parameters->SetPixe(false);
  auto* manager=G4RunManagerFactory::CreateRunManager(G4RunManagerType::Serial);
  manager->SetUserInitialization(new SynDetector);manager->SetUserInitialization(new SynPhysics);manager->Initialize();
  G4LossTableManager::Instance()->SetAtomDeexcitation(nullptr);G4DNAChemistryManager::Instance()->Initialize();
  auto* previous=G4StateManager::GetStateManager()->GetExceptionHandler();SynExceptions exceptions;
  auto* table=G4DNAMolecularReactionTable::GetReactionTable();
  G4DiffusionControlledReactionModel model;model.SetReactionTable(table);
  G4rsSynMakeReaction placement;placement.SetReactionTable(table);placement.SetReactionModel(&model);
  fprintf(file,"# schema: syn_paired_v1; Geant4: 1141; claim: finite-input mapping only\n");
  fprintf(file,"# seed: %llu; tapes: %llu; source_sha256: 8ae87604437c1f7d57f410961c01d51adf9215e5a35f6702247ad6592a1f0559\n",seed,tapes);
  fprintf(file,"kind,channel,ratio,dt_ps,orientation,fixed,tape,zx,zy,zz,u_phi,u_theta,u_product,mask,status,a_nm,d1_nm2_ps,d2_nm2_ps,p1x,p1y,p1z,p2x,p2y,p2z,r1x,r1y,r1z,r2x,r2y,r2z,products,px,py,pz,killed\n");
  size_t rows=0;
  auto emit=[&](const char* kind,int id,double ratio,double dt,int orientation,int fixed,bool coincident) {
    auto* reaction=table->GetReaction(id);
    auto* confA=const_cast<G4MolecularConfiguration*>(reaction->GetReactant1());
    auto* confB=const_cast<G4MolecularConfiguration*>(reaction->GetReactant2());
    double a=reaction->GetEffectiveReactionRadius();double originalA=confA->GetDiffusionCoefficient(),originalB=confB->GetDiffusionCoefficient();
    if(fixed==0) {confA->UnFinalize();confA->SetDiffusionCoefficient(0.);confA->Finalize();}
    if(fixed==1) {confB->UnFinalize();confB->SetDiffusionCoefficient(0.);confB->Finalize();}
    G4ThreeVector origin=orientation==2?G4ThreeVector(10*a,-7*a,3*a):G4ThreeVector();
    G4ThreeVector delta=orientation==1?G4ThreeVector(0,0,ratio*a):G4ThreeVector(ratio*a,0,0);
    if(coincident)delta={};
    for(unsigned tape=0;tape<tapes;++tape) {
      SynTape(seed,tape);
      auto first=std::unique_ptr<G4Track>((new G4Molecule(confA))->BuildTrack(0,origin));
      auto second=std::unique_ptr<G4Track>((new G4Molecule(confB))->BuildTrack(0,origin+delta));
      bool error=false;std::unique_ptr<G4ITReactionChange> change;
      placement.TestReactibility(*first,*second,dt*ps,false);
      try {
        if(std::string(kind)=="base") change=placement.MakeReaction(*first,*second);
        else placement.UpdatePositionForReaction(*first,*second);
      }catch(const SynCoincidence&) {error=true;}
      if(error!=(std::string(kind)=="coincident")) throw std::runtime_error("unexpected coincidence branch");
      unsigned expected=(std::string(kind)=="base" && dt>0)?31:0;
      if(change && reaction->GetNbProducts()>0)expected|=32;
      if(synInputs.mask!=expected)throw std::runtime_error("unexpected semantic draw mask");
      fprintf(file,"%s,%d,%.17g,%.17g,%d,%d,%u,%.17g,%.17g,%.17g,%.17g,%.17g,%.17g,%u,%s,%.17g,%.17g,%.17g",kind,id,ratio,dt,orientation,fixed,tape,synInputs.zx,synInputs.zy,synInputs.zz,synInputs.phi,synInputs.theta,synInputs.product,synInputs.mask,error?"coincident_error":"ok",a/nm,confA->GetDiffusionCoefficient()/(nm*nm/ps),confB->GetDiffusionCoefficient()/(nm*nm/ps));
      SynVector(file,origin);SynVector(file,origin+delta);
      if(error)fprintf(file,",NA,NA,NA,NA,NA,NA");else{SynVector(file,first->GetPosition());SynVector(file,second->GetPosition());}
      if(change) {
        if(change->GetNumberOfSecondaries()!=reaction->GetNbProducts() || !change->WereParentsKilled())throw std::runtime_error("product/parent-removal mismatch");
        fprintf(file,",%d",change->GetNumberOfSecondaries());
        if(change->GetNumberOfSecondaries()) {
          auto site=change->GetSecondary(0)->GetPosition();
          for(int i=0;i<change->GetNumberOfSecondaries();++i) {
            if(change->GetSecondary(i)->GetPosition()!=site || GetMolecule(change->GetSecondary(i))->GetMolecularConfiguration()!=reaction->GetProduct(i))throw std::runtime_error("product placement/order mismatch");
          }
          SynVector(file,site);
        }else fprintf(file,",NA,NA,NA");
        fprintf(file,",1\n");
      }else fprintf(file,",NA,NA,NA,NA,NA\n");
      // Erase through the owning list so EraseListNode detaches the molecule's
      // node before deleting it. Clear() on a populated delayed list invokes
      // its destructor, leaving G4IT::TakeOutBox a stale node to delete again.
      if(change) for(int i=0;i<change->GetNumberOfSecondaries();++i) {
        auto* product=change->GetSecondary(i);
        auto* list=G4TrackList::GetList(product);
        if(!list)throw std::runtime_error("source product missing owning list");
        list->erase(product);
      }
      change.reset();G4ITTrackHolder::Instance()->Clear();++rows;
    }
    if(fixed==0) {confA->UnFinalize();confA->SetDiffusionCoefficient(originalA);confA->Finalize();}
    if(fixed==1) {confB->UnFinalize();confB->SetDiffusionCoefficient(originalB);confB->Finalize();}
  };
  for(int id:{1,2,11})for(double ratio:{1.,3.})for(double dt:{0.,1.,1000.})for(int orientation:{0,1,2})emit("base",id,ratio,dt,orientation,-1,false);
  for(int fixed:{0,1})for(double dt:{0.,1.,1000.})for(bool coincident:{false,true})emit("immobile",2,coincident?0.:3.,dt,0,fixed,coincident);
  for(int id:{1,2,11})for(double dt:{0.,1.,1000.})emit("coincident",id,0.,dt,0,-1,true);
  if(rows!=75*tapes)throw std::runtime_error("incomplete scenario grid");
  if(ftell(file)<0 || ftell(file)>999800)throw std::runtime_error("fixture exceeds one MB budget");
  fprintf(file,"# complete: true; seed: %llu; tapes: %llu; rows: %zu\n",seed,tapes,rows);
  if(fclose(file))throw std::runtime_error("output close failure");
  G4StateManager::GetStateManager()->SetExceptionHandler(previous);delete manager;return 0;
 }catch(const std::exception& e) {fprintf(stderr,"syn paired: %s; incomplete output is not evidence\n",e.what());return 1;}
}
