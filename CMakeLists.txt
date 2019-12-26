cmake_minimum_required(VERSION 3.1)
project(hltas)

include(CTest)

set(CMAKE_MODULE_PATH ${CMAKE_MODULE_PATH} "${CMAKE_CURRENT_SOURCE_DIR}/cmake/")
enable_language(Rust)
include(CMakeCargo)

add_subdirectory(hltas-cpp-bridge)
add_subdirectory(cpp)

target_link_libraries(hltas-cpp INTERFACE hltas-cpp-bridge)
target_link_libraries(hltas-cpp-bridge INTERFACE hltas-cpp)
