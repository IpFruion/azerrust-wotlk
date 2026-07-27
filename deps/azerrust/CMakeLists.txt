cmake_minimum_required(VERSION 3.22)

project(azerrust)

set(CMAKE_CXX_STANDARD 20)

set(CARGO_MANIFEST ${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml)
set(CARGO_BUILD_CMD cargo build --manifest-path ${CARGO_MANIFEST} --release)
set(CARGO_TARGET_DIR ${CMAKE_CURRENT_BINARY_DIR}/target)

set(AZERRUST_SOURCE_FILES
    ${CMAKE_CURRENT_SOURCE_DIR}/src/lib.rs
    ${CMAKE_CURRENT_SOURCE_DIR}/src/conditions/mod.rs)
set(AZERRUST_BRIDGE_CPP ${CARGO_TARGET_DIR}/cxxbridge/azerrust)

set(AZERRUST_HEADER ${CMAKE_CURRENT_BINARY_DIR}/include/azerrust.h)
set(AZERRUST_HELPERS_HEADER ${CMAKE_CURRENT_BINARY_DIR}/include/azerrust_helpers.h)

set(AZERRUST_LIB ${CARGO_TARGET_DIR}/release/${CMAKE_STATIC_LIBRARY_PREFIX}azerrust${CMAKE_STATIC_LIBRARY_SUFFIX})

find_program(CARGO_CMD cargo)

if (NOT CARGO_CMD)
    message(FATAL_ERROR "Need to have cargo installed to build azerrust")
endif()

# Re-find boost in this scope (find_package vars don't cross sibling
# add_subdirectory boundaries).
find_package(Boost QUIET)

# Build extra CXXFLAGS from system deps (boost etc.) for the cargo/cxx-build
# compilation that happens during `cargo build`.
set(CARGO_CXXFLAGS "")
set(AZERRUST_EXTRA_INCLUDES "")
foreach(dir ${Boost_INCLUDE_DIRS})
    if(CARGO_CXXFLAGS)
        string(APPEND CARGO_CXXFLAGS " ")
    endif()
    string(APPEND CARGO_CXXFLAGS "-I${dir}")
    string(APPEND AZERRUST_EXTRA_INCLUDES ":${dir}")
endforeach()

set(AZERRUST_HELPERS_HEADER_SOURCE ${CMAKE_CURRENT_SOURCE_DIR}/include/azerrust_helpers.h)

# Run cargo to build the Rust library and generate bridge headers
add_custom_command(
    OUTPUT  ${AZERRUST_HEADER} ${AZERRUST_HELPERS_HEADER} ${AZERRUST_LIB}
    COMMAND CXXBRIDGE_OUTPUT_DIR=${CARGO_TARGET_DIR}/cxxbridge CARGO_TARGET_DIR=${CARGO_TARGET_DIR} CXXFLAGS="${CARGO_CXXFLAGS}" AZERRUST_EXTRA_INCLUDE_DIRS="${AZERRUST_EXTRA_INCLUDES}" ${CARGO_BUILD_CMD}
    COMMAND mkdir -p ${CMAKE_CURRENT_BINARY_DIR}/include
    COMMAND cp ${AZERRUST_HELPERS_HEADER_SOURCE} ${AZERRUST_HELPERS_HEADER}
    COMMAND cat ${AZERRUST_BRIDGE_CPP}/src/lib.rs.h ${AZERRUST_BRIDGE_CPP}/src/conditions/mod.rs.h > ${AZERRUST_HEADER}
    COMMAND echo "#include <azerrust_conditions.h>" >> ${AZERRUST_HEADER}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    DEPENDS ${AZERRUST_SOURCE_FILES} ${AZERRUST_HELPERS_HEADER_SOURCE}
    USES_TERMINAL
    COMMENT "Running Cargo..."
)

# Custom target to trigger the cargo build
add_custom_target(azerrust_cargo_build DEPENDS ${AZERRUST_HEADER} ${AZERRUST_HELPERS_HEADER} ${AZERRUST_LIB})

# Interface library: provides include dirs and link deps, but no compiled code.
# The Rust .a (which contains both Rust object code and cxx bridge C++ code)
# is linked directly by the worldserver target.
add_library(azerrust INTERFACE)
add_dependencies(azerrust azerrust_cargo_build)
target_include_directories(azerrust INTERFACE ${CMAKE_CURRENT_BINARY_DIR}/include/)
target_link_libraries(azerrust INTERFACE pthread dl)

set_target_properties(azerrust PROPERTIES FOLDER "deps")
