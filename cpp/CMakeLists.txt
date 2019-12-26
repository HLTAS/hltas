add_library(hltas-cpp src/hltas.cpp)
set_property(TARGET hltas-cpp PROPERTY CXX_STANDARD 14)
target_include_directories(hltas-cpp PUBLIC src)

add_executable(tests src/tests.cpp)
set_property(TARGET tests PROPERTY CXX_STANDARD 14)
target_include_directories(tests SYSTEM PRIVATE catch2)
target_link_libraries(tests hltas-cpp)

include(catch2/Catch.cmake)
catch_discover_tests(tests WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
