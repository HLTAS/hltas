cmake_minimum_required (VERSION 3.1)
project (hltas)

set (CMAKE_CXX_STANDARD 14)

find_package (Boost 1.57.0 REQUIRED)
include_directories (SYSTEM ${Boost_INCLUDE_DIRS})

add_library (hltas src/hltas.cpp)

add_executable (tests src/tests.cpp)
target_include_directories (tests SYSTEM PRIVATE catch2)
target_link_libraries (tests hltas)
if (NOT MSVC)
	target_link_libraries (tests pthread)
endif (NOT MSVC)
include (CTest)
include (catch2/Catch.cmake)
catch_discover_tests (tests WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
