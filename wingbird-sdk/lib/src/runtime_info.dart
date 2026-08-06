import 'dart:developer';
import 'dart:io';

import 'package:device_info_plus/device_info_plus.dart';
import 'package:flutter/services.dart';
import 'package:package_info_plus/package_info_plus.dart';

class RuntimeInfo {
  final String architecture;
  final String platform;
  final String appVersion;
  final String libAppPath;

  RuntimeInfo({required this.architecture, required this.platform, required this.appVersion, required this.libAppPath});
}

final MethodChannel _channel = MethodChannel('com.wingbird.sdk/native');

Future<String?> _getLibAppPath() async {
  try {
    final String? path = await _channel.invokeMethod<String>('getLibAppPath');
    return path;
  } on PlatformException catch (e) {
    log('Failed to get libapp.so path: ${e.message}');
    return null;
  }
}

Future<RuntimeInfo> getRuntimeInfo() async {
  final deviceInfoPlugin = DeviceInfoPlugin();
  final packageInfo = await PackageInfo.fromPlatform();

  final appVersion = '${packageInfo.version}+${packageInfo.buildNumber}';
  final libAppPath = await _getLibAppPath();

  if (libAppPath == null) {
    throw StateError('Failed to get libapp.so path');
  }

  switch (Platform.operatingSystem) {
    case 'android':
      final androidInfo = await deviceInfoPlugin.androidInfo;
      final architecture = androidInfo.supportedAbis.isNotEmpty
          ? androidInfo.supportedAbis.first
          : '';
      if (architecture.isEmpty) {
        throw StateError('Failed to get device architecture');
      }
      return RuntimeInfo(architecture: architecture, platform: 'android', appVersion: appVersion, libAppPath: libAppPath);
    default:
      throw StateError('Unsupported platform');
  }
}
