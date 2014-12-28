cmake_minimum_required (VERSION 3.1)
project (hltas)

set (BOOST_DIR "./boost" CACHE PATH "Path to the base boost folder (where boost, doc, libs and other folders are situated).")
include_directories(${BOOST_DIR})

add_library(hltas
	src/hltas.cpp)
