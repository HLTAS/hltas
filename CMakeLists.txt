cmake_minimum_required (VERSION 3.1)
project (hltas)

set (BOOST_DIR "./boost" CACHE PATH "Path to the base boost folder (where boost, doc, libs and other folders are situated).")
include_directories (${BOOST_DIR})

set (ADDITIONAL_CXX_FLAGS "")
if (NOT MSVC)
	set (ADDITIONAL_CXX_FLAGS "${ADDITIONAL_CXX_FLAGS} -std=c++11")
endif ()

set (CMAKE_CXX_FLAGS  "${CMAKE_CXX_FLAGS} ${ADDITIONAL_CXX_FLAGS}")

add_library (hltas
	src/hltas.cpp)
