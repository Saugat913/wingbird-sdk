# Wingbird SDK

Flutter plugin for integrating Wingbird runtime patching into Flutter applications.

> **Status:** Experimental / beta. Android integration is the current focus; other platforms are scaffolded or commented out in the plugin configuration.

## What it does

The SDK:

1. Initializes the generated Flutter Rust Bridge library.
2. Checks compile-time configuration for `WINGBIRD_SERVER_URL` and `WINGBIRD_APP_ID`.
3. Exits as a no-op when the required build-time configuration is absent.
4. Reads runtime information such as application version, platform, architecture, and the Flutter `libapp` path.
5. Creates a patch-manager configuration using the application support directory.
6. Reads the current patch number and starts the Rust patch manager.

## Usage

Initialize the SDK early in the Flutter application lifecycle:

```dart
import 'package:wingbird_sdk/sdk.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  await Wingbird.init(channel: Channel.prod);

  runApp(const MyApp());
}
```

The package currently expects these values to be supplied as Dart compile-time environment variables:

```bash
flutter build apk \
  --dart-define=WINGBIRD_SERVER_URL=https://your-server.example \
  --dart-define=WINGBIRD_APP_ID=your-app-id
```

Use `Channel.stage` for staging configuration when appropriate. The server URL and app ID are read from `String.fromEnvironment`, so they must be supplied during the Flutter build.

## Architecture

- Dart API in `lib/`
- Generated Rust bindings through Flutter Rust Bridge
- Rust patch manager under `rust/`
- Native Flutter plugin packaging with Android FFI support
- Runtime and filesystem integration through Flutter platform packages

## Development

```bash
flutter pub get
flutter analyze
flutter test
```

Rust bridge generation and native builds depend on the repository's Flutter Rust Bridge and Cargokit configuration. Review the generated bindings and platform-specific directories before extending support to additional platforms.

## Related repositories

- [wingbird](https://github.com/Saugat913/wingbird) — Rust CLI
- [wingbird-server](https://github.com/Saugat913/wingbird-server) — backend and dashboard
- [wingbird-backup](https://github.com/Saugat913/wingbird-backup) — original consolidated repository

## License

Apache-2.0
