cmake_minimum_required(VERSION 3.16...3.27)
project(g4rs_ref)

# Geant4 11.4.1 with the DNA physics constructors (same requirement as the extended/medical/dna
# examples). Configure with  -DGeant4_DIR=<install>/lib/cmake/Geant4  (or source geant4.sh).
find_package(Geant4 11.4 REQUIRED)
include(${Geant4_USE_FILE})

add_executable(g4rs_ref g4rs_ref.cc)
target_link_libraries(g4rs_ref ${Geant4_LIBRARIES})

# 900 keV chemistry events overflowed the default 1 MB Windows stack once (exit 0xC00000FD, 2026-10-16,
# physchem production run); give the main thread 512 MB. No effect on the physics.
if(MSVC)
  target_link_options(g4rs_ref PRIVATE /STACK:536870912)
endif()
