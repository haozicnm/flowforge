# Project-level configuration.
cmake_minimum_required(VERSION 3.13)
project(flowforge_desktop LANGUAGES CXX)

set(BINARY_NAME "flowforge")
set(APPLICATION_ID "com.flowforge.flowforge")

# Standard Flutter tooling.
include(flutter/CMakeLists.txt OPTIONAL RESULT_VARIABLE FLUTTER_CMAKE_FILE)
if(NOT FLUTTER_CMAKE_FILE)
  message(FLUTTER_CMAKE_FILE "Flutter CMake file not found. Flutter desktop support may need to be enabled.")
endif()

# Linux platform configuration.
add_subdirectory(${FLUTTER_MANAGED_DIR})

find_package(PkgConfig REQUIRED)
pkg_check_modules(GTK REQUIRED IMPORTED_TARGET gtk+-3.0)

add_definitions(-DAPPLICATION_ID="${APPLICATION_ID}")

add_executable(${BINARY_NAME}
  "main.cc"
  "my_application.cc"
  "${FLUTTER_MANAGED_DIR}/generated_plugin_registrant.cc"
)

apply_standard_settings(${BINARY_NAME})
target_link_libraries(${BINARY_NAME} PRIVATE flutter)
target_link_libraries(${BINARY_NAME} PRIVATE PkgConfig::GTK)

target_include_directories(${BINARY_NAME} PRIVATE
  "${CMAKE_SOURCE_DIR}"
)

add_dependencies(${BINARY_NAME} flutter_assemble)
