cmake_minimum_required(VERSION 3.1)
project(hltas)

include(CTest)

set(CMAKE_MODULE_PATH ${CMAKE_MODULE_PATH} "${CMAKE_CURRENT_SOURCE_DIR}/cmake/")
enable_language(Rust)
include(CMakeCargo)

add_subdirectory(hltas-cpp-bridge)
add_subdirectory(cpp)

# This should probably be "NEW", but that's not available until CMake 3.13.
# OLD seems to work fine.
if(POLICY CMP0079)
    cmake_policy(SET CMP0079 OLD)
endif(POLICY CMP0079)

target_link_libraries(hltas-cpp INTERFACE hltas-cpp-bridge)
target_link_libraries(hltas-cpp-bridge INTERFACE hltas-cpp)
