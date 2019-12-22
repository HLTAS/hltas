cmake_minimum_required(VERSION 3.1)
project(hltas)

include(CTest)



find_package(Threads REQUIRED)
find_package(Boost 1.57.0 REQUIRED)

add_library(hltas src/hltas.cpp)
set_property(TARGET hltas PROPERTY CXX_STANDARD 14)
target_include_directories(hltas SYSTEM PRIVATE ${Boost_INCLUDE_DIRS})
target_link_libraries(hltas INTERFACE Threads::Threads)

add_executable(tests src/tests.cpp)
set_property(TARGET tests PROPERTY CXX_STANDARD 14)
target_include_directories(tests SYSTEM PRIVATE catch2)
target_link_libraries(tests hltas)

include(catch2/Catch.cmake)
catch_discover_tests(tests WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
