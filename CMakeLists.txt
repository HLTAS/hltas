cmake_minimum_required (VERSION 3.1)
project (hltas)

include_directories (${BOOST_ROOT})

set (CMAKE_CXX_STANDARD 11)

add_library (hltas src/hltas.cpp)
