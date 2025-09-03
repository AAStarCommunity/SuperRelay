# SPDX-License-Identifier: BSD-2-Clause
#
# Copyright (c) 2025, AAStarCommunity
# SuperRelay Trusted Application CMake Configuration

cmake_minimum_required(VERSION 3.10)

project(SuperRelayTA C)

# TA UUID and binary name
set(TA_UUID "12345678-5b69-11d4-9fee-00c04f4c3456")
set(BINARY "${TA_UUID}")

# Source files
set(SRCS
    super_relay_ta.c
)

# Include directories
include_directories(include)

# Compiler flags
add_compile_options(
    -Wall
    -Wextra
    -Werror
    -Os
    -g3
)

add_compile_definitions(
    TRACE_LEVEL=TRACE_INFO
    CFG_TEE_TA_LOG_LEVEL=2
)

# OP-TEE TA development kit integration
if(DEFINED ENV{TA_DEV_KIT_DIR})
    set(TA_DEV_KIT_DIR $ENV{TA_DEV_KIT_DIR})
    message(STATUS "Using TA_DEV_KIT_DIR: ${TA_DEV_KIT_DIR}")
    
    # Include TA development kit
    include(${TA_DEV_KIT_DIR}/mk/cmake/ta.cmake)
    
    # Add TA target
    add_ta_target(
        TARGET ${BINARY}
        SOURCES ${SRCS}
        UUID ${TA_UUID}
    )
    
else()
    message(FATAL_ERROR "TA_DEV_KIT_DIR environment variable not set")
endif()

# Install target
install(FILES ${CMAKE_BINARY_DIR}/${BINARY}.ta
        DESTINATION lib/optee_armtz/
        PERMISSIONS OWNER_READ OWNER_WRITE GROUP_READ WORLD_READ)

# Custom targets
add_custom_target(info
    COMMAND echo "SuperRelay Trusted Application"
    COMMAND echo "UUID: ${TA_UUID}"
    COMMAND echo "Sources: ${SRCS}"
    COMMAND echo "TA Dev Kit: ${TA_DEV_KIT_DIR}"
)