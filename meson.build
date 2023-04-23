project('hltas', ['rust', 'cpp'],
  version : '0.8.0',
  default_options : ['rust_std=2021', 'cpp_std=c++14'],
)

rust = import('rust')

nom_dep = dependency('nom-rs', version : ['>= 7.1.3', '< 8'])
cookie_factory_dep = dependency('cookie-factory-rs', version : ['>= 0.3.2', '< 0.4'])

hltas = static_library('hltas', 'src/lib.rs',
  rust_crate_type : 'rlib',
  dependencies : [nom_dep, cookie_factory_dep],
  pic : true,
)

hltas_dep = declare_dependency(link_with : hltas,
  dependencies : [nom_dep, cookie_factory_dep],
)
meson.override_dependency('hltas-rs', hltas_dep)

rust.test('hltas-tests', hltas,
  workdir : meson.project_source_root(),
)

subdir('hltas-cpp-bridge')
subdir('cpp')
