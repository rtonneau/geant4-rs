cmake_minimum_required(VERSION 3.16...3.27)
project(g4rs_ref)

# Geant4 11.4.1 with the DNA physics constructors (same requirement as the extended/medical/dna
# examples). Configure with  -DGeant4_DIR=<install>/lib/cmake/Geant4  (or source geant4.sh).
find_package(Geant4 11.4 REQUIRED)
include(${Geant4_USE_FILE})

add_executable(g4rs_ref g4rs_ref.cc)
target_link_libraries(g4rs_ref ${Geant4_LIBRARIES})

# Optional paired diagnostic. No machine source path is committed; configure
# explicitly with the read-only normative source root after verifying its version.
set(G4RS_SYN_SOURCE_ROOT "" CACHE PATH "Read-only Geant4 11.4.1 source root for paired diagnostics")
if(G4RS_SYN_SOURCE_ROOT)
  set(syn_make_source "${G4RS_SYN_SOURCE_ROOT}/source/processes/electromagnetic/dna/models/src/G4DNAMakeReaction.cc")
  file(SHA256 "${syn_make_source}" syn_make_hash)
  if(NOT syn_make_hash STREQUAL "8ae87604437c1f7d57f410961c01d51adf9215e5a35f6702247ad6592a1f0559")
    message(FATAL_ERROR "IRT-syn source hash mismatch; semantic draw labels are version-pinned")
  endif()
  add_executable(g4rs_syn_ref SynPaired.cc)
  target_link_libraries(g4rs_syn_ref ${Geant4_LIBRARIES})
  target_compile_definitions(g4rs_syn_ref PRIVATE G4RS_SYN_MAKE_SOURCE="${syn_make_source}")
endif()

# 900 keV chemistry events overflowed the default 1 MB Windows stack once (exit 0xC00000FD, 2026-10-16,
# physchem production run); give the main thread 512 MB. No effect on the physics.
if(MSVC)
  target_link_options(g4rs_ref PRIVATE /STACK:536870912)
endif()

# Separate ordinary installed-library trace; no interception of model code or RNG.
add_executable(g4rs_syn_scheduler_ref SynScheduler.cc)
target_link_libraries(g4rs_syn_scheduler_ref ${Geant4_LIBRARIES})

add_executable(g4rs_syn_scheduler_followup SynSchedulerFollowup.cc)
target_link_libraries(g4rs_syn_scheduler_followup ${Geant4_LIBRARIES})
