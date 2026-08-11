library;

import 'package:path_provider/path_provider.dart';
import 'package:sdk/src/runtime_info.dart';
import 'package:sdk/src/rust/api/patch.dart';
import 'package:sdk/src/rust/frb_generated.dart';

enum Channel { prod, stage }

class Wingbird {
  Wingbird._({
    required this.serverUrl,
    required this.appId,
    required this.channel,
  });

  final String serverUrl;
  final String appId;
  final Channel channel;

  static Future<void> init({required Channel channel}) async {
    print('[Wingbird SDK] Initializing RustLib...');
    await RustLib.init();
    print('[Wingbird SDK] RustLib initialized successfully.');

    const enabled =
        bool.hasEnvironment("WINGBIRD_SERVER_URL") &&
        bool.hasEnvironment("WINGBIRD_APP_ID");

    print('[Wingbird SDK] Update check enabled: $enabled');

    if (!enabled) return;

    const serverUrl = String.fromEnvironment("WINGBIRD_SERVER_URL");
    const appId = String.fromEnvironment("WINGBIRD_APP_ID");
    print(
      '[Wingbird SDK] Config - Server: $serverUrl, AppId: $appId, Channel: $channel',
    );

    final runtimeInfo = await getRuntimeInfo();
    print(
      '[Wingbird SDK] RuntimeInfo - Version: ${runtimeInfo.appVersion}, Platform: ${runtimeInfo.platform}, Arch: ${runtimeInfo.architecture}, LibPath: ${runtimeInfo.libAppPath}',
    );

    final supportDir = await getApplicationSupportDirectory();

    final patchManagerConfig = WingbirdPatchManagerConfig(
      serverUrl: serverUrl,
      appId: appId,
      version: runtimeInfo.appVersion,
      channel: channel.name,
      platform: runtimeInfo.platform,
      architecture: runtimeInfo.architecture,
      rootPath: supportDir.path,
      nativeLibDir: supportDir.path,
    );

    final patchManager = await WingbirdPatchManager.newInstance(
      config: patchManagerConfig,
    );
    final currentPatchNumber = await patchManager.getCurrentPatchNumber();
    print('[Wingbird SDK] Current patch number: $currentPatchNumber');
  }
}
