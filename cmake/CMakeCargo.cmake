function(cargo_build)
    cmake_parse_arguments(CARGO "" "NAME" "" ${ARGN})
    string(REPLACE "-" "_" LIB_NAME ${CARGO_NAME})

    set(CARGO_TARGET_DIR ${CMAKE_CURRENT_BINARY_DIR})

    if(WIN32)
        if(CMAKE_SIZEOF_VOID_P EQUAL 8)
            set(LIB_TARGET "x86_64-pc-windows-msvc")
        else()
            set(LIB_TARGET "i686-pc-windows-msvc")
        endif()
	elseif(ANDROID)
        if(ANDROID_SYSROOT_ABI STREQUAL "x86")
            set(LIB_TARGET "i686-linux-android")
        elseif(ANDROID_SYSROOT_ABI STREQUAL "x86_64")
            set(LIB_TARGET "x86_64-linux-android")
        elseif(ANDROID_SYSROOT_ABI STREQUAL "arm")
            set(LIB_TARGET "arm-linux-androideabi")
        elseif(ANDROID_SYSROOT_ABI STREQUAL "arm64")
            set(LIB_TARGET "aarch64-linux-android")
        endif()
    elseif(IOS)
		set(LIB_TARGET "universal")
    elseif(CMAKE_SYSTEM_NAME STREQUAL Darwin)
        set(LIB_TARGET "x86_64-apple-darwin")
	else()
        if(CMAKE_SIZEOF_VOID_P EQUAL 8)
            set(LIB_TARGET "x86_64-unknown-linux-gnu")
        else()
            set(LIB_TARGET "i686-unknown-linux-gnu")
        endif()
    endif()

    if(NOT CMAKE_BUILD_TYPE)
        set(LIB_BUILD_TYPE "debug")
    elseif(${CMAKE_BUILD_TYPE} STREQUAL "Release")
        set(LIB_BUILD_TYPE "release")
    else()
        set(LIB_BUILD_TYPE "debug")
    endif()

    set(LIB_FILE "${CARGO_TARGET_DIR}/${LIB_TARGET}/${LIB_BUILD_TYPE}/${CMAKE_STATIC_LIBRARY_PREFIX}${LIB_NAME}${CMAKE_STATIC_LIBRARY_SUFFIX}")

    set(CARGO_ARGS "")

	if(IOS)
		set(CARGO_COMMAND "lipo")
	else()
    	set(CARGO_COMMAND "build")
		list(APPEND CARGO_ARGS "--target" ${LIB_TARGET})
	endif()

    if(${LIB_BUILD_TYPE} STREQUAL "release")
        list(APPEND CARGO_ARGS "--release")
    endif()

    file(GLOB_RECURSE LIB_SOURCES "*.rs")

    set(CARGO_ENV_COMMAND ${CMAKE_COMMAND} -E env "CARGO_TARGET_DIR=${CARGO_TARGET_DIR}")

    add_custom_command(
        OUTPUT ${LIB_FILE}
        COMMAND ${CARGO_ENV_COMMAND} ${CARGO_EXECUTABLE} ARGS ${CARGO_COMMAND} ${CARGO_ARGS}
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
        DEPENDS ${LIB_SOURCES}
        COMMENT "Building Rust library")
    add_custom_target(${CARGO_NAME}_target ALL DEPENDS ${LIB_FILE})
    add_library(${CARGO_NAME} STATIC IMPORTED GLOBAL)
    add_dependencies(${CARGO_NAME} ${CARGO_NAME}_target)
    set_target_properties(${CARGO_NAME} PROPERTIES IMPORTED_LOCATION ${LIB_FILE})

    add_test(${CARGO_NAME} ${CARGO_ENV_COMMAND} ${CARGO_EXECUTABLE} test ${CARGO_ARGS})
    set_tests_properties(${CARGO_NAME} PROPERTIES WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})

    file(WRITE ${CARGO_TARGET_DIR}/rustc_lib_test.rs "")
    execute_process(
        COMMAND ${RUSTC_EXECUTABLE} --crate-type=staticlib --print=native-static-libs --out-dir=${CARGO_TARGET_DIR} ${CARGO_TARGET_DIR}/rustc_lib_test.rs
        OUTPUT_VARIABLE RUST_LINK_LIBRARIES_OUT
        ERROR_VARIABLE RUST_LINK_LIBRARIES_ERR
        RESULT_VARIABLE RUSTC_RET)
    if (NOT "${RUSTC_RET}" STREQUAL "0")
        message(FATAL_ERROR "rustc failed: ${RUST_LINK_LIBRARIES_ERR}")
    endif()
    set(RUST_LINK_LIBRARIES "${RUST_LINK_LIBRARIES_OUT} ${RUST_LINK_LIBRARIES_ERR}")
    string(REGEX MATCHALL "note: native-static-libs: ([\-a-zA-Z_0-9 \.]+)" RUST_LINK_LIBRARIES "${RUST_LINK_LIBRARIES}")
    string(REPLACE "note: native-static-libs: " "" RUST_LINK_LIBRARIES "${RUST_LINK_LIBRARIES}")
    if (WIN32)
        message(STATUS "Removing ms vcrt library from list")
        string(REPLACE "msvcrt.lib" "" RUST_LINK_LIBRARIES "${RUST_LINK_LIBRARIES}")
    endif ()
    separate_arguments(RUST_LINK_LIBRARIES)
    message(STATUS "Rust libraries: ${RUST_LINK_LIBRARIES}")
    target_link_libraries(${CARGO_NAME} INTERFACE ${RUST_LINK_LIBRARIES})
endfunction()
