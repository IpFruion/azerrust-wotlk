cmake_minimum_required(VERSION 3.22)
project(azerrust_geometry)
set(CMAKE_CXX_STANDARD 20)

set(CARGO_MANIFEST ${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml)
set(CARGO_BUILD_CMD cargo build --manifest-path ${CARGO_MANIFEST} --release)
set(CARGO_TARGET_DIR ${CMAKE_CURRENT_BINARY_DIR}/target)

set(AZERRUST_GEOMETRY_BRIDGE_CPP ${CARGO_TARGET_DIR}/cxxbridge/azerrust-geometry)
set(AZERRUST_GEOMETRY_HEADER ${CMAKE_CURRENT_BINARY_DIR}/include/azerrust_geometry.h)
set(AZERRUST_GEOMETRY_LIB ${CARGO_TARGET_DIR}/release/${CMAKE_STATIC_LIBRARY_PREFIX}azerrust_geometry${CMAKE_STATIC_LIBRARY_SUFFIX})

find_program(CARGO_CMD cargo)
if (NOT CARGO_CMD)
    message(FATAL_ERROR "Need to have cargo installed to build azerrust_geometry")
endif()

add_custom_command(
    OUTPUT  ${AZERRUST_GEOMETRY_HEADER} ${AZERRUST_GEOMETRY_LIB}
    COMMAND CXXBRIDGE_OUTPUT_DIR=${CARGO_TARGET_DIR}/cxxbridge CARGO_TARGET_DIR=${CARGO_TARGET_DIR} ${CARGO_BUILD_CMD}
    COMMAND mkdir -p ${CMAKE_CURRENT_BINARY_DIR}/include
    COMMAND cp ${AZERRUST_GEOMETRY_BRIDGE_CPP}/src/lib.rs.h ${AZERRUST_GEOMETRY_HEADER}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/src/lib.rs
    USES_TERMINAL
    COMMENT "Running Cargo (azerrust_geometry)..."
)

add_custom_target(azerrust_geometry_cargo_build
    DEPENDS ${AZERRUST_GEOMETRY_HEADER} ${AZERRUST_GEOMETRY_LIB})

add_library(azerrust_geometry INTERFACE)
add_dependencies(azerrust_geometry azerrust_geometry_cargo_build)
target_include_directories(azerrust_geometry INTERFACE ${CMAKE_CURRENT_BINARY_DIR}/include/)

set_target_properties(azerrust_geometry PROPERTIES FOLDER "deps")
