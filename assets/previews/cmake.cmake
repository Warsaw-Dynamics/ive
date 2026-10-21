# Build script for the IVE demo
cmake_minimum_required(VERSION 3.20)
project(ivedemo LANGUAGES C CXX)

#[[ Bracket comment:
    options below can be set with -D ]]
option(IVE_WITH_TESTS "Build the tests" ON)
set(IVE_SOURCES src/main.cpp src/editor.cpp)
set(CMAKE_CXX_STANDARD 20 CACHE STRING "C++ standard")

find_package(Threads REQUIRED)

function(ive_add_tool name source)
  add_executable(${name} ${source})
  target_link_libraries(${name} PRIVATE Threads::Threads)
  message(STATUS "Tool ${name} from $ENV{HOME}\tready")
endfunction()

macro(ive_log text)
  message([=[log: ${text} is not expanded here]=])
endmacro()

if(IVE_WITH_TESTS AND NOT WIN32)
  ive_add_tool(ive-tests tests/main.cpp)
elseif(DEFINED IVE_LEGACY)
  ive_log("legacy build")
else()
  foreach(source IN LISTS IVE_SOURCES)
    list(APPEND IVE_OBJECTS "${source}.o")
  endforeach()
endif()
