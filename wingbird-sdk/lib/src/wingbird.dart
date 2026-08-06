library;

import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';
import 'package:sdk/src/runtime_info.dart';
import 'package:sdk/src/rust/api/patch.dart' as patch_api;
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

  static Wingbird? _instance;

  static Future<void> init({required Channel channel}) async {
    await RustLib.init();

    const enabled =
        bool.hasEnvironment("WINGBIRD_SERVER_URL") &&
        bool.hasEnvironment("WINGBIRD_APP_ID");

    if (enabled) {
      final serverUrl = String.fromEnvironment("WINGBIRD_SERVER_URL");
      final appId = String.fromEnvironment("WINGBIRD_APP_ID");

      _instance = Wingbird._(
        serverUrl: serverUrl,
        appId: appId,
        channel: channel,
      );

      final runtimeInfo = await getRuntimeInfo();

      final tempDir = await getTemporaryDirectory();

      final supportDir = await getApplicationSupportDirectory();
      final wingbirdDir = Directory(p.join(supportDir.path, 'wingbird'));
      await wingbirdDir.create(recursive: true);

      final downloadPath = p.join(tempDir.path, 'latest.patch');
      final finalVersionFilePath = p.join(wingbirdDir.path, 'libapp.so');

      await _instance!._update(
        currentVersion: runtimeInfo.appVersion,
        architecture: runtimeInfo.architecture,
        platform: runtimeInfo.platform,
        downloadPath: downloadPath,
        initialVersionFilePath: runtimeInfo.libAppPath,
        finalVersionFilePath: finalVersionFilePath,
      );
    }
  }

 
  Future<bool> _downloadPatch({
    required String currentVersion,
    required String architecture,
    required String platform,
    required String downloadPath,
  }) {
    return patch_api.downloadPatch(
      serverUrl: serverUrl,
      appId: appId,
      channel: channel.name,
      currentVersion: currentVersion,
      architecture: architecture,
      platform: platform,
      downloadPath: downloadPath,
    );
  }


  Future<void> _applyPatch({
    required String patchFilePath,
    required String initialVersionFilePath,
    required String finalVersionFilePath,
  }) {
    return patch_api.applyPatch(
      patchFilePath: patchFilePath,
      initialVersionFilePath: initialVersionFilePath,
      finalVersionFilePath: finalVersionFilePath,
    );
  }

  Future<bool> _update({
    required String currentVersion,
    required String architecture,
    required String platform,
    required String downloadPath,
    required String initialVersionFilePath,
    required String finalVersionFilePath,
  }) async {
    final downloaded = await _downloadPatch(
      currentVersion: currentVersion,
      architecture: architecture,
      platform: platform,
      downloadPath: downloadPath,
    );

    if (!downloaded) {
      return false;
    }

    try {
      await _applyPatch(
        patchFilePath: downloadPath,
        initialVersionFilePath: initialVersionFilePath,
        finalVersionFilePath: finalVersionFilePath,
      );
    } catch (_) {
      final patchFile = File(downloadPath);
      if (await patchFile.exists()) {
        await patchFile.delete();
      }
      rethrow;
    }

    return true;
  }
}
