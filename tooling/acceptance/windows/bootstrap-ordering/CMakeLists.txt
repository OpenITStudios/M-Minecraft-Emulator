cmake_minimum_required(VERSION 3.24)
project(BootstrapOrderingSelfTest LANGUAGES CXX)

option(BOOTSTRAP_ORDERING_TEST_ONLY "Build the standalone self-child ordering experiment" OFF)
if(NOT BOOTSTRAP_ORDERING_TEST_ONLY)
  message(FATAL_ERROR "Explicit BOOTSTRAP_ORDERING_TEST_ONLY=ON is required")
endif()
if(NOT WIN32 OR NOT MSVC OR NOT CMAKE_SIZEOF_VOID_P EQUAL 8)
  message(FATAL_ERROR "This standalone probe requires Windows MSVC x64")
endif()
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
set(DETOURS_ROOT "" CACHE PATH "Existing trusted Microsoft Detours checkout; never downloaded")
find_package(Git REQUIRED)
execute_process(COMMAND "${GIT_EXECUTABLE}" -C "${DETOURS_ROOT}" rev-parse HEAD
  OUTPUT_VARIABLE detours_head OUTPUT_STRIP_TRAILING_WHITESPACE RESULT_VARIABLE head_result)
if(NOT head_result EQUAL 0 OR NOT detours_head STREQUAL "1dbd4ea281d73d478aa8d09a8b83fbd265f715f7")
  message(FATAL_ERROR "DETOURS_ROOT must be the reviewed Detours commit")
endif()
execute_process(COMMAND "${GIT_EXECUTABLE}" -C "${DETOURS_ROOT}" diff --quiet HEAD -- src
  RESULT_VARIABLE dirty_result)
execute_process(COMMAND "${GIT_EXECUTABLE}" -C "${DETOURS_ROOT}" ls-files --others --exclude-standard src
  OUTPUT_VARIABLE untracked RESULT_VARIABLE untracked_result)
if(NOT dirty_result EQUAL 0 OR NOT untracked_result EQUAL 0 OR NOT untracked STREQUAL "")
  message(FATAL_ERROR "Detours src must be clean and have no untracked source")
endif()

# Separate test-only library; no vendoring, download, prebuilt DLL or product linkage.
set(detours_sources detours modules disasm image creatwth
  disolx86 disolx64 disolia64 disolarm disolarm64)
foreach(source IN LISTS detours_sources)
  list(APPEND detours_files "${DETOURS_ROOT}/src/${source}.cpp")
endforeach()
add_library(probe-detours STATIC ${detours_files})
target_include_directories(probe-detours PUBLIC "${DETOURS_ROOT}/src")
target_compile_definitions(probe-detours PRIVATE DETOUR_DEBUG=0 WIN32_LEAN_AND_MEAN)
target_compile_options(probe-detours PRIVATE /W4 /WX)

add_library(bootstrap-dependency SHARED dependency.cpp)
add_library(bootstrap-compat SHARED compat.cpp compat.def)
target_link_libraries(bootstrap-compat PRIVATE probe-detours shell32 ole32 uuid advapi32)
add_executable(bootstrap-ordering probe.cpp)
target_link_libraries(bootstrap-ordering PRIVATE bootstrap-dependency probe-detours shell32 ole32 uuid advapi32)
add_dependencies(bootstrap-ordering bootstrap-compat)
foreach(target bootstrap-ordering bootstrap-dependency bootstrap-compat)
  target_compile_definitions(${target} PRIVATE UNICODE _UNICODE WIN32_LEAN_AND_MEAN NOMINMAX
    _WIN32_WINNT=0x0A00)
  target_compile_options(${target} PRIVATE /W4 /WX /permissive- /EHsc)
endforeach()
# Deliberately no install, CTest registration, product dependency or automatic execution.
