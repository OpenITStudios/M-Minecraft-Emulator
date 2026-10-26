cmake_minimum_required(VERSION 3.23)
project(MivubiIsolationAbiProbe LANGUAGES CXX)

# A standalone read-only ABI tool, never a driver/service installer.
if(NOT WIN32 OR NOT MSVC OR CMAKE_SIZEOF_VOID_P LESS 8)
  message(FATAL_ERROR "Isolation ABI probe requires 64-bit MSVC on Windows.")
endif()
if(MSVC_VERSION LESS 1930)
  message(FATAL_ERROR "Visual Studio 2022 MSVC or newer is required.")
endif()

set(MIVUBI_PRIVATE_FORK_ROOT "" CACHE PATH
  "Separate upstream-history-preserving fork checkout with MIVUBI source patch")
set(PINNED_COMMIT "abd4c240031cfb46666daa74ca47ed4e48b561e4")
set(UPSTREAM_ROOT "${CMAKE_CURRENT_LIST_DIR}/../isolation-upstream/Sandboxie")
set(OWNED_PRIVATE_SOURCE OFF)
if(NOT "${MIVUBI_PRIVATE_FORK_ROOT}" STREQUAL "")
  set(UPSTREAM_ROOT "${MIVUBI_PRIVATE_FORK_ROOT}")
  set(OWNED_PRIVATE_SOURCE ON)
endif()
if(NOT EXISTS "${UPSTREAM_ROOT}/Sandboxie/common/my_version.h")
  message(FATAL_ERROR "Initialize pinned submodule or prepare separate private fork source first.")
endif()

find_package(Git REQUIRED)
if(OWNED_PRIVATE_SOURCE)
  # Use the same exact committed seven-file source verifier as private MSBuild
  # and payload staging. Ancestor + source marker alone is insufficient:
  # it accepts unauthorized additional edits and even an uncommitted patch.
  find_program(MIVUBI_NODE_EXECUTABLE NAMES node.exe node REQUIRED)
  set(PRIVATE_SOURCE_VERIFIER
    "${CMAKE_CURRENT_LIST_DIR}/fork-overlay/apply-private-source.mjs")
  execute_process(
    COMMAND "${MIVUBI_NODE_EXECUTABLE}" "${PRIVATE_SOURCE_VERIFIER}"
      --verify-committed --fork-root "${UPSTREAM_ROOT}"
    RESULT_VARIABLE PRIVATE_SOURCE_RESULT
    OUTPUT_VARIABLE PRIVATE_SOURCE_EVIDENCE
    ERROR_VARIABLE PRIVATE_SOURCE_ERROR
    OUTPUT_STRIP_TRAILING_WHITESPACE
  )
  if(NOT PRIVATE_SOURCE_RESULT STREQUAL "0")
    message(FATAL_ERROR
      "Private ABI probe source is not a clean exact committed fork: ${PRIVATE_SOURCE_ERROR}")
  endif()
  message(STATUS
    "Private ABI probe uses exact committed source verified by canonical Node owner")

else()
  # Unmodified upstream mode still requires exact pinned HEAD.
  execute_process(
    COMMAND "${GIT_EXECUTABLE}" -C "${UPSTREAM_ROOT}" rev-parse HEAD
    OUTPUT_VARIABLE UPSTREAM_SHA
    OUTPUT_STRIP_TRAILING_WHITESPACE
    RESULT_VARIABLE GIT_RESULT
  )
  if(NOT GIT_RESULT EQUAL 0 OR
     NOT UPSTREAM_SHA STREQUAL "${PINNED_COMMIT}")
    message(FATAL_ERROR "ABI source must match pinned upstream release.")
  endif()
  # HEAD alone is insufficient if tracked source files were modified after
  # checkout. Include untracked inputs, but ignore ordinary build outputs as
  # prescribed by the pinned upstream .gitignore.
  execute_process(
    COMMAND "${GIT_EXECUTABLE}" -C "${UPSTREAM_ROOT}"
      status --porcelain=v1 --untracked-files=all
    OUTPUT_VARIABLE UPSTREAM_DIRTY
    OUTPUT_STRIP_TRAILING_WHITESPACE
    RESULT_VARIABLE UPSTREAM_STATUS_RESULT
  )
  if(NOT UPSTREAM_STATUS_RESULT EQUAL 0 OR
     NOT "${UPSTREAM_DIRTY}" STREQUAL "")
    message(FATAL_ERROR "Pinned upstream ABI source checkout is dirty.")
  endif()
endif()

add_executable(mivubi-isolation-abi-probe src/abi_probe.cpp)
target_compile_features(mivubi-isolation-abi-probe PRIVATE cxx_std_17)
target_include_directories(mivubi-isolation-abi-probe PRIVATE "${UPSTREAM_ROOT}/Sandboxie")
target_compile_definitions(mivubi-isolation-abi-probe PRIVATE
  UNICODE _UNICODE)
if(OWNED_PRIVATE_SOURCE)
  target_compile_definitions(mivubi-isolation-abi-probe PRIVATE MIVUBI_EXPECT_PRIVATE_SOURCE=1)
endif()
target_link_libraries(mivubi-isolation-abi-probe PRIVATE advapi32 wintrust)
target_compile_options(mivubi-isolation-abi-probe PRIVATE /W4 /WX /sdl)

enable_testing()
add_test(NAME source_abi_contract_selftest
  COMMAND mivubi-isolation-abi-probe --selftest)
