scalable_dir = 'hicolor' / 'scalable' / 'apps'

install_data(
  scalable_dir / '@0@.svg'.format(application_id),
  install_dir: iconsdir / scalable_dir
)

symbolic_dir = 'hicolor' / 'symbolic' / 'apps'

install_data(
  symbolic_dir / '@0@-symbolic.svg'.format(base_id),
  install_dir: iconsdir / symbolic_dir,
  rename: '@0@-symbolic.svg'.format(application_id)
)
