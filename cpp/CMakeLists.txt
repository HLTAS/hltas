find_package(Threads REQUIRED)
find_package(Boost 1.57.0 REQUIRED)

add_library(hltas-cpp src/hltas.cpp)
set_property(TARGET hltas-cpp PROPERTY CXX_STANDARD 14)
target_include_directories(hltas-cpp SYSTEM PRIVATE ${Boost_INCLUDE_DIRS})
target_include_directories(hltas-cpp PUBLIC src)
target_link_libraries(hltas-cpp INTERFACE Threads::Threads)

add_executable(tests src/tests.cpp)
set_property(TARGET tests PROPERTY CXX_STANDARD 14)
target_include_directories(tests SYSTEM PRIVATE catch2)
target_link_libraries(tests hltas-cpp)

include(catch2/Catch.cmake)
catch_discover_tests(tests WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
