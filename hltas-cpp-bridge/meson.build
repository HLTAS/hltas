cfg_if_dep = dependency('cfg-if-rs', version : ['>= 1.0.0', '< 2'])

hltas_cpp_bridge = static_library('hltas_cpp_bridge', 'src/lib.rs',
  rust_crate_type : 'staticlib',
  dependencies : [hltas_dep, nom_dep, cfg_if_dep],
  pic : true,
)

hltas_cpp_bridge_dep = declare_dependency(link_with : hltas_cpp_bridge)
meson.override_dependency('hltas-cpp-bridge', hltas_cpp_bridge_dep)

rust.test('hltas-cpp-bridge-tests', hltas_cpp_bridge)
