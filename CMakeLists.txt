cmake_minimum_required (VERSION 3.1)
project (hltas)

set (CMAKE_CXX_STANDARD 14)

find_package (Boost 1.57.0 REQUIRED)
include_directories (SYSTEM ${Boost_INCLUDE_DIRS})

add_library (hltas src/hltas.cpp)
