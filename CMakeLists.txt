cmake_minimum_required(VERSION 3.1)
project(hltas)

include(CTest)

set(CMAKE_MODULE_PATH ${CMAKE_MODULE_PATH} "${CMAKE_CURRENT_SOURCE_DIR}/cmake/")
enable_language(Rust)
include(CMakeCargo)

cargo_build(NAME hltas)

add_subdirectory(cpp)

target_link_libraries(hltas-cpp INTERFACE hltas)
target_link_libraries(hltas INTERFACE hltas-cpp)
