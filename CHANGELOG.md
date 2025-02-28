# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Planned]

- Update dependencies to later versions.
- Miri CI.

## [0.1.0-alpha.4] - UNRELEASED

### Added

- Testing framework.
- Serial port.

### Changed

### Removed

## [0.1.0-alpha.3] - 2025-02-28

### Added

- `println!` VGA write functionality.
- `panic!` messages.
- API for getting and setting VGA attributes.

### Removed

- Old `vga_text` API, no more `print_str` and `write!`.

## [0.1.0-alpha.2] - 2025-02-28

### Added

- Generated version message.
- `print_str` VGA write function.
- `write!` VGA macro.
- On-the-fly changing of VGA colours.

### Changed

- Crate organisation. Package now contains `tlenek_core` library crate and `tlenek_bin` binary crate.

### Removed

- `putc_vga_text` in favour of safer and more flexible `print_str` and `write!`.

## [0.1.0-alpha.1] - 2025-02-27

### Added

- Initial release.
- Display OS name and version message.
