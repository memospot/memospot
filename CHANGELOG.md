<!-- markdownlint-disable blanks-around-headings blanks-around-lists no-duplicate-heading -->

# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!--
Types of changes
----------------
Added: for new features.
Changed: for changes in existing functionality.
Deprecated: for soon-to-be removed features.
Removed: for now removed features.
Fixed: for any bug fixes.
Security: in case of vulnerabilities.
-->

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Changed

- (Memos) Bundled version: v0.25.2.

### Fixed

- (macOS) "Fix" issues with dialogs using window.confirm() not showing. This fix is a side effect of Memos now using a modal for the confirmation dialog. The issue may return in the future if the upstream changes are reverted, as the JS API bug is in Tauri itself.

### Added

- Allow zooming the config window - ([5370ee7](https://github.com/memospot/memospot/commit/5370ee7fdac578e8c3aac757616fb47c89d91a98)).
- Reduce animation setting - ([e0965de](https://github.com/memospot/memospot/commit/e0965de7410929776f9491dfdd4a59675a06d8ee)).
- (Linux/macOS) Gracefully shut down Memos via SIGINT - ([b77000e](https://github.com/memospot/memospot/commit/b77000ee4f8ed6142a641960f322ba0b542dd544)).
- (Windows) Gracefully shut down Memos via CTRL+BREAK - ([938be92](https://github.com/memospot/memospot/commit/938be92f6ca0e8cc744c2b95d408dc0bbd8c88c9)).
- Improved logging. If enabled, Memos server output will be logged to `memos.log`. Delete any existing `logging_config.yaml` to get the new logger configuration.

## [1.0.0] - 2025-02-21

### Changed

- (Memos) Bundled version: v0.24.0.
- Ported to Tauri v2.
- Update contribution guide - ([9068f58](https://github.com/memospot/memospot/commit/9068f583d478bda798686e931ada1442f8886673)).
- Database checkpoint and config writing are now more robust and less resource-intensive.

### Added

- Menu bar with useful resources.
- Configuration window.
- Initial localization support.
- Support for a custom user agent when running in client mode.
- (Linux) Heuristics to improve the WebView rendering.
- (macOS) Allow plain HTTP remote servers - ([65d5c08](https://github.com/memospot/memospot/commit/65d5c0862005e2400435ca918996ba1ca907837c)).

### Fixed

- Fix file drag-and-drop.
- (macOS) Fix issue with Memos not launching.

### Known issues

- (Linux) Tauri v2 degraded WebView performance on some systems with an NVIDIA GPU due to a regression in WebKitGTK.
  The app is usable, but there is little we can do to make the rendering fast.

## [0.1.7] - 2024-09-11

### Changed

- (Memos) Bundled version: v0.22.5.
- Disable UPX on Windows - ([3bbd1f0](https://github.com/memospot/memospot/commit/3bbd1f0e9d64e44914a73e11100fe6a2b8931204)).

### Added

- Auto-updater.

- Restore the last window size upon restart. Auto-centering is enabled by default but can be disabled via the settings file. - ([46e23ff](https://github.com/memospot/memospot/commit/46e23ff86aa492575dd6d38db624b76cda1409fe)).

## [0.1.6] - 2024-09-03

### Changed

- (Memos) Bundled version: v0.22.4.

### Added

- Display app version and remote URL on the title bar - ([9eed232](https://github.com/memospot/memospot/commit/9eed232d38e019a57636ffa1f08a23d3f251b141)).
- (Linux)/(macOS) Optional support for a data directory under `$XDG_CONFIG_HOME/memospot` - ([52b2e06](https://github.com/memospot/memospot/commit/52b2e06137034645164d7b4ff5dd71a0d02f871a)).
- (Memos) Handle new environment variables `password_auth` and `public` - ([93db74b](https://github.com/memospot/memospot/commit/93db74bd7268648ccc8185ec5065d995eecf031f)).
- (Memos) Handle new environment variable `instance_url` - ([934366e](https://github.com/memospot/memospot/commit/934366e69777205e6f314dd6ddf305fbc575652a)).
- (Linux) New RPM package.
- Support external Memos server - ([c354557](https://github.com/memospot/memospot/commit/c354557192ad52ee400384f31a57cd6295e11783)):
  Sample `memospot.yaml` file:

  ```yaml
  remote:
    enabled: true
    url: https://demo.usememos.com/
  ```

## [0.1.5] - 2024-05-25

### Changed

- (Memos) Bundled version: v0.22.0.

### Fixed

- (Memos) Fixed WebView CORS issue for v0.22+.

### Added

- (Memos) Add v0.22 storage settings migration (Memos ignores previous storage settings on v0.22) - ([9e8f4a2](https://github.com/memospot/memospot/commit/9e8f4a29ee57996c0688e7612a805c7a190a1916)).

- Compatibilized asset migrations with both Memos <= v0.21.1 and >= v0.22.0.

### Known issues

- (Memos) The tag management system has changed. Your tag list will be empty.
  You must hover the "Tags" section and click the refresh icon 🔁 to get your tags back.

## [0.1.4] - 2024-02-27

- (Memos) Bundled version: v0.20.0.

### Fixed

- (Windows) Timezone error in statistics API request - ([1a0755c](https://github.com/memospot/memospot/commit/1a0755c2de3a037d349575edc5994e168fbbb0d5)).

## [0.1.3] - 2024-02-26

### Changed

- (Memos) Bundled version: v0.20.0.
- Updated UI wording and color scheme.

### Added

- (Memos) Handle Memos versions with loose front-end files (sidecar dist folder; v0.18.2+) - ([a385616](https://github.com/memospot/memospot/commit/a385616897b57e43e1a6f93f78256355a252e1fb)).
- (Memos) Disable Memos metrics (metrics were removed from Memos v0.20+).
- Add database checkpoint on closing: ensures changed data is persisted.
- Database migrator:
  It makes the database and assets portable between any system.
  The internal resource path migration will run only once when you first start this new version. It will create a new table `memospot_migrations` to keep track of migrations. This won't affect Memos at all.

  The path migrator can migrate 300k resources in less than 30 seconds on a modern machine. If you see the "Something went wrong" message, just wait a minute and click the "Check again" button.

  This feature was added to address issues with the built-in Memos database migration system, which was really slow to the point the related migration was completely removed at some point.

- Database backup system:
  Runs automatically before database migrations. Backups are created using a zip file container, but compression is in Zstandard format (due to a better speed/compression ratio). If needed, you can decompress those files with WinRAR, 7-Zip-zstd, or p7zip.

- (Memos) Lots of new settings: allows using a custom folder for Memos data and allows fine-grained control of the server.

  Sample `memospot.yaml` file:

  ```yaml
  memos:
    # Use this to spawn a custom Memos binary.
    binary_path: null

    # Memos working dir. This is where the `dist` folder must reside.
    working_dir: null

    # Memos data storage. Where the database and assets are stored.
    data: null

    # Mode: [prod] | dev | demo. This affects the database used on startup.
    mode: prod

    # Address where Memos will listen for connections.
    addr: 127.0.0.1

    # Memos port. Managed by Memospot. You can set a custom port, but it
    # will be automatically changed if the port is in use at Memospot startup.
    port: 0

    # Custom environment variables for Memos. Custom keys will be automatically uppercased and prefixed with "MEMOS_". Make sure to always quote custom env values, so they get parsed as strings.
    # env:
    #   NEW_ENV_VAR: "my value"
    env: null
  memospot:
    backups:
      # Enable backups [true]. Currently, backups only run before
      # database migrations and there's no retention management.
      enabled: true
      # Backup directory.
      path: null
    migrations:
      # Enable migrations [true]. Currently, there's one migration available that will change local resource paths from absolute to relative, making your data fully portable.
      enabled: true
    log:
      # Enable logging [false]. Used for advanced debugging.
      # A new file called `logging_config.yaml` will be created next to this file.
      # You can change `root.level` from `info` to `debug` to increase the logging level.
      enabled: true
  ```

### Removed

- (Memos) No more output logging: log is now available only for debugging Memospot itself.

## [0.1.2] - 2023-12-25

### Changed

- (Memos) Bundled version: v0.18.1

- (Windows) UPX is no longer used; this should reduce false-positive detections by AV software.

### Added

- (Memos) Server port is now persisted via the `memospot.yaml` settings file:
  The port will only change if it's in use during a next application start. This will increase WebView cache reuse and circumvent an issue that arose with Memos v0.18.0, where the user's language and theme are not retrieved from the database.

- (Windows) Custom installers:
  Selecting "Delete the application data" on uninstaller will delete Memospot WebView cache, Memos and Memospot binaries, but never the database, assets or settings.
  This will also prevent undesired behavior in case Tauri upstream changes.

- A 'Try again' button will appear if the Memos server takes too long to respond:
  This allows the server to take its time to load when running in low-end or heavily loaded systems.

## [0.1.1] - 2023-11-28

- (Memos) Bundled version: v0.17.1.

### Fixed

- (Windows) Resolve proper data directory on Windows ([#5](https://github.com/memospot/memospot/issues/5)) - ([aa6bf92](https://github.com/memospot/memospot/commit/aa6bf925161efb45abdd83f7c7dc4650186e2399))

- (Linux) Add GStreamer to the AppImage bundle.

## [0.1.0] - 2023-11-27

- (Memos) Bundled version: v0.17.1.

### Known issues

- (Linux)(AppImage): If a video is uploaded, it may not play or the WebView may hang. This is due to GStreamer not being bundled. It will be fixed in the next release. This does not affect the deb package (as long as you have GStreamer installed).

<!-- next-url -->

[Unreleased]: https://github.com/memospot/memospot/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/memospot/memospot/releases/tag/v1.0.0
[0.1.7]: https://github.com/memospot/memospot/releases/tag/v0.1.7
[0.1.6]: https://github.com/memospot/memospot/releases/tag/v0.1.6
[0.1.5]: https://github.com/memospot/memospot/releases/tag/v0.1.5
[0.1.4]: https://github.com/memospot/memospot/releases/tag/v0.1.4
[0.1.3]: https://github.com/memospot/memospot/releases/tag/v0.1.3
[0.1.2]: https://github.com/memospot/memospot/releases/tag/v0.1.2
[0.1.1]: https://github.com/memospot/memospot/releases/tag/v0.1.1
[0.1.0]: https://github.com/memospot/memospot/releases/tag/v0.1.0
