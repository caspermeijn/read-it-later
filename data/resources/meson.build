# Resources
ui_config = configuration_data()
ui_config.set('app-id', application_id)
ui_config.set('version', meson.project_version() + version_suffix)
ui_config.set('name-suffix', name_suffix)
ui_preconfigured_files = files(
  'ui/login.ui.in',
  'ui/window.ui.in',
)
ui_dependencies = []
foreach ui_file: ui_preconfigured_files
  ui_dependencies += configure_file(
    input:  ui_file,
    output: '@BASENAME@',
    configuration: ui_config
  )
endforeach

resources = gnome.compile_resources(
  'resources',
  'resources.gresource.xml',
  gresource_bundle: true,
  source_dir: meson.current_build_dir(),
  install: true,
  install_dir: pkgdatadir,
)
