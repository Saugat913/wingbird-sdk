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

## Installation

Add Wingbird SDK to your Flutter application's `pubspec.yaml`:

```yaml
dependencies:
  wingbird_sdk:
    git:
      url: https://github.com/Saugat913/wingbird-sdk.git
```

Install the dependency:

```bash
flutter pub get
```

## Usage

### 1. Initialize the SDK

Initialize Wingbird before starting your application:

```dart
import 'package:flutter/widgets.dart';
import 'package:wingbird_sdk/sdk.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  await Wingbird.init(
    channel: Channel.prod,
  );

  runApp(const MyApp());
}
```

Use `Channel.stage` when working with a staging environment.

### 2. Build and publish with Wingbird CLI

The Wingbird SDK works together with the **Wingbird CLI** to deliver application patches.

Install the CLI by following the instructions in the [Wingbird repository](https://github.com/Saugat913/wingbird).

Use the CLI to:

* Build and register application releases.
* Generate binary patches between releases.
* Upload patches to your Wingbird server.
* Manage patch versions and deployments.

Refer to the [Wingbird CLI documentation](https://github.com/Saugat913/wingbird) for setup and usage instructions.

> **Important:** Installing the Flutter SDK alone is not sufficient for patch delivery. Your application must be configured and released through the Wingbird CLI.

## Related repositories

- [wingbird](https://github.com/Saugat913/wingbird) — Rust CLI
- [wingbird-server](https://github.com/Saugat913/wingbird-server) — backend and dashboard
- [wingbird-backup](https://github.com/Saugat913/wingbird-backup) — original consolidated repository

## License

Apache-2.0
