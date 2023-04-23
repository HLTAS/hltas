hltas_cpp = static_library('hltas_cpp', 'src/hltas.cpp',
  dependencies : [hltas_cpp_bridge_dep],
  pic : true,
)

extra_winlibs = []
if host_machine.system() == 'windows'
  cpp = meson.get_compiler('cpp')
  extra_winlibs += [cpp.find_library('userenv')]
  extra_winlibs += [cpp.find_library('ws2_32')]
  extra_winlibs += [cpp.find_library('bcrypt')]
endif

hltas_cpp = declare_dependency(link_with : hltas_cpp,
  include_directories : 'src',
  dependencies : extra_winlibs,
)
meson.override_dependency('hltas-cpp', hltas_cpp)

catch2 = declare_dependency(include_directories : 'catch2')

hltas_cpp_tests = executable('hltas-cpp-tests', 'src/tests.cpp',
  dependencies : [hltas_cpp, catch2],
)
test('hltas-cpp-tests', hltas_cpp_tests,
  workdir : meson.current_source_dir(),
)