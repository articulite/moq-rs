cmake_minimum_required(VERSION 3.12)
project(MoqNativePlugin)

# Set C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Add MoQ libraries
# This assumes you've built the MoQ libraries already and they're available
if(WIN32)
    set(MOQ_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release")
elseif(APPLE)
    set(MOQ_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/lib/macos")
elseif(ANDROID)
    set(MOQ_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/lib/android/${ANDROID_ABI}")
else()
    set(MOQ_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/lib/linux")
endif()

# Include directories
include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

# Define library
add_library(MoqNativePlugin SHARED
    MoqNativePlugin.cpp
    MoqNativePlugin.h
)

# Link against MoQ libraries
if(WIN32)
    target_link_libraries(MoqNativePlugin
        ${MOQ_LIB_DIR}/libmoq_transfork.rlib
        ${MOQ_LIB_DIR}/libmoq_karp.rlib
        ${MOQ_LIB_DIR}/libmoq_native.rlib
    )
else()
    target_link_libraries(MoqNativePlugin
        ${MOQ_LIB_DIR}/libmoq_transfork.a
        ${MOQ_LIB_DIR}/libmoq_karp.a
        ${MOQ_LIB_DIR}/libmoq_native.a
    )
endif()

# Set output directory
set_target_properties(MoqNativePlugin PROPERTIES
    LIBRARY_OUTPUT_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/bin/${CMAKE_SYSTEM_NAME}"
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/bin/${CMAKE_SYSTEM_NAME}"
)

# Platform-specific settings
if(WIN32)
    # Windows-specific settings
    set_target_properties(MoqNativePlugin PROPERTIES
        SUFFIX ".dll"
    )
elseif(APPLE)
    # macOS-specific settings
    set_target_properties(MoqNativePlugin PROPERTIES
        SUFFIX ".dylib"
        PREFIX "lib"
    )
elseif(ANDROID)
    # Android-specific settings
    set_target_properties(MoqNativePlugin PROPERTIES
        SUFFIX ".so"
        PREFIX "lib"
    )
else()
    # Linux-specific settings
    set_target_properties(MoqNativePlugin PROPERTIES
        SUFFIX ".so"
        PREFIX "lib"
    )
endif() 