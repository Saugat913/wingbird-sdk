package com.flutter_rust_bridge.sdk

import android.content.ContentProvider
import android.content.ContentValues
import android.database.Cursor
import android.net.Uri
import android.util.Log
import io.flutter.FlutterInjector
import io.flutter.embedding.engine.loader.FlutterLoader
import java.io.File
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build

class WingbirdInitProvider : ContentProvider() {
  override fun onCreate(): Boolean {
    val ctx = context?.applicationContext ?: return true
    val patchFile = File(ctx.filesDir, "wingbird/lib/libapp.so")
    val versionFile = File(ctx.filesDir, "wingbird/lib/.version")

    val patchPath = if (isValidPatch(patchFile) && isVersionMatching(ctx, versionFile)) {
      patchFile.absolutePath
    } else {
      null
    }

    Log.i("wingbirdProvider", "Initializing WingbirdInitProvider...")

    val loader = FlutterInjector.instance().flutterLoader()
    loader.startInitialization(ctx, FlutterLoader.Settings())

    val args = mutableListOf<String>()
    if (patchPath != null) {
      Log.i("wingbirdProvider", "Loaded valid patched libapp.so at: $patchPath")
      args.add("--aot-shared-library-name=$patchPath")
    } else {
      Log.i("wingbirdProvider", "No valid patch found, using default libapp.so")
    }
    loader.ensureInitializationComplete(ctx, args.toTypedArray())
    return true
  }

  private fun isValidPatch(file: File): Boolean =
    file.exists() && file.canRead() && file.length() > 0

  private fun isVersionMatching(ctx: Context, versionFile: File): Boolean {
    return try {
      if (!versionFile.exists()) return false
      val parts = versionFile.readText().trim().split(":")
      if (parts.size != 2) return false
      parts[0] == getAppVersion(ctx)
    } catch (e: Exception) {
      Log.w("wingbirdProvider", "Failed to read patch version file: ${e.message}")
      false
    }
  }

  private fun getAppVersion(context: Context): String {
    return try {
      val packageInfo = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        context.packageManager.getPackageInfo(
          context.packageName,
          PackageManager.PackageInfoFlags.of(0)
        )
      } else {
        @Suppress("DEPRECATION")
        context.packageManager.getPackageInfo(context.packageName, 0)
      }
      "${packageInfo.versionName}+${packageInfo.versionCode}"
    } catch (e: PackageManager.NameNotFoundException) {
      ""
    }
  }

  override fun query(u: Uri, p: Array<String>?, s: String?, sa: Array<String>?, so: String?): Cursor? = null
  override fun getType(u: Uri): String? = null
  override fun insert(u: Uri, v: ContentValues?): Uri? = null
  override fun delete(u: Uri, s: String?, sa: Array<String>?): Int = 0
  override fun update(u: Uri, v: ContentValues?, s: String?, sa: Array<String>?): Int = 0
}
