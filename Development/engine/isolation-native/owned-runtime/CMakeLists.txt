cmake_minimum_required(VERSION 3.16)
project(MivubiOwnedStartPolicy LANGUAGES C CXX)

# Real C++ policy test for the SAME source that is inlined into the forked
# Sandboxie Start.exe; runs without Windows SDK, a driver, accounts or GitHub.
add_executable(mivubi-owned-start-policy-test
  mivubi_owned_start_policy.test.cpp)
set_target_properties(mivubi-owned-start-policy-test PROPERTIES
  CXX_STANDARD 17 CXX_STANDARD_REQUIRED YES CXX_EXTENSIONS NO)
if(MSVC)
  target_compile_options(mivubi-owned-start-policy-test PRIVATE /W4 /WX)
else()
  target_compile_options(mivubi-owned-start-policy-test PRIVATE
    -Wall -Wextra -Werror -pedantic)
endif()

enable_testing()
add_test(NAME owned_start_policy_native
  COMMAND mivubi-owned-start-policy-test)

# The driver-facing marker must be a portable C definition, not a C++-only
# shim: the pinned upstream kernel API implementation is compiled from api.c.
add_executable(mivubi-core-identity-test mivubi_core_identity.test.c)
set_target_properties(mivubi-core-identity-test PROPERTIES
  C_STANDARD 11 C_STANDARD_REQUIRED YES C_EXTENSIONS NO)
if(MSVC)
  target_compile_options(mivubi-core-identity-test PRIVATE /W4 /WX)
else()
  target_compile_options(mivubi-core-identity-test PRIVATE
    -Wall -Wextra -Werror -pedantic)
endif()
add_test(NAME owned_core_identity_native COMMAND mivubi-core-identity-test)

# The same C policy is inserted into the real private Sandboxie kernel's
# Box_Alloc path. Keep this native test active in Release (not assert()).
add_executable(mivubi-kernel-box-policy-test mivubi_kernel_box_policy.test.c)
set_target_properties(mivubi-kernel-box-policy-test PROPERTIES
  C_STANDARD 11 C_STANDARD_REQUIRED YES C_EXTENSIONS NO)
if(MSVC)
  target_compile_options(mivubi-kernel-box-policy-test PRIVATE /W4 /WX)
else()
  target_compile_options(mivubi-kernel-box-policy-test PRIVATE
    -Wall -Wextra -Werror -pedantic)
endif()
add_test(NAME owned_kernel_box_policy_native
  COMMAND mivubi-kernel-box-policy-test)
