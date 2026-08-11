# wingbird_sdk

Wingbird hot-fix patching SDK for Flutter apps. Loads an  `libapp.so` patch at runtime without going through the app store.

## Usage

```yaml
dependencies:
  wingbird_sdk:
    path: path/to/wingbird-sdk
```

```dart
import 'package:wingbird_sdk/sdk.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await Wingbird.init(channel: Channel.prod);
  runApp(const MyApp());
}
```

Config is read from `--dart-define` build flags:
- `WINGBIRD_SERVER_URL` — server base URL
- `WINGBIRD_APP_ID` — app id registered on the server
- `Channel.prod` vs `Channel.stage` is selected in code

## How it works

`Wingbird.init` downloads any available patch for the current release, verifies it, and hot-swaps `libapp.so` on the next launch. The native side is generated with `flutter_rust_bridge`; re-run `flutter_rust_bridge_codegen generate` in `rust/` after changing Rust sources.

## Prebuilt vs local native build

- **Prebuilt** (default): CI builds `libsdk.so` for each architecture and publishes it as a signed prebuilt artifact; the app fetches it automatically. See `.github/workflows/cargokit-precompile.yaml`.
- **Local**: if no prebuilt artifact matches, `cargokit` falls back to building native code from `rust/` during `flutter build`.

## License

Apache License 2.0 — see `LICENSE`.
