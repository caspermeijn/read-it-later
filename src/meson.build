global_conf = configuration_data()
global_conf.set_quoted('APP_ID', application_id)
global_conf.set_quoted('PKGDATADIR', prefix / pkgdatadir)
global_conf.set_quoted('PROFILE', profile)
global_conf.set_quoted('NAME_SUFFIX', name_suffix)
global_conf.set_quoted('VERSION', version + version_suffix)
global_conf.set_quoted('GETTEXT_PACKAGE', gettext_package)
global_conf.set_quoted('LOCALEDIR', localedir)
config = configure_file(
    input: 'config.rs.in',
    output: 'config.rs',
    configuration: global_conf
)
# Copy the config.rs output to the source directory.
run_command(
  'cp',
  join_paths(meson.build_root(), 'src', 'config.rs'),
  join_paths(meson.source_root(), 'src', 'config.rs'),
  check: true
)

# include_bytes! only takes a string literal
resource_conf = configuration_data()
resource_conf.set_quoted('RESOURCEFILE', resources.full_path())
resource_rs = configure_file(
  input: 'static_resources.rs.in',
  output: 'static_resources.rs',
  configuration: resource_conf
)

run_command(
  'cp',
  resource_rs,
  meson.current_source_dir(),
  check: true
)

sources = files(
  'models/article.rs',
  'models/articles.rs',
  'models/client_manager.rs',
  'models/mod.rs',
  'models/object_wrapper.rs',
  'models/preview_image.rs',
  'models/secret_service.rs',
  'settings/key.rs',
  'settings/mod.rs',
  'settings/settings_manager.rs',
  'views/archive.rs',
  'views/article.rs',
  'views/favorites.rs',
  'views/login.rs',
  'views/mod.rs',
  'views/unread.rs',
  'widgets/articles/index.rs',
  'widgets/articles/list.rs',
  'widgets/articles/mod.rs',
  'widgets/articles/preview.rs',
  'widgets/articles/row.rs',
  'widgets/login.rs',
  'widgets/mod.rs',
  'widgets/settings.rs',
  'widgets/window.rs',
  'application.rs',
  'config.rs',
  'database.rs',
  'main.rs',
  'schema.rs',
  'static_resources.rs',
  'utils.rs',
  'window_state.rs',
)

custom_target('cargo-build',
  build_by_default: true,
  input: sources,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: bindir,
  depends: resources,
  command: [
    cargo_script,
    meson.build_root(),
    meson.source_root(),
    '@OUTPUT@',
    localedir,
    profile
  ]
)

