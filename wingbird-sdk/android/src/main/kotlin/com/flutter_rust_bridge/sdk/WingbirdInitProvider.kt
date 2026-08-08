package com.flutter_rust_bridge.sdk

import android.content.ContentProvider
import android.content.ContentValues
import android.database.Cursor
import android.net.Uri
import android.util.Log
import io.flutter.FlutterInjector
import io.flutter.embedding.engine.loader.FlutterLoader
import java.io.File

class WingbirdInitProvider : ContentProvider() {
  override fun onCreate(): Boolean {
    val ctx = context?.applicationContext ?: return true
    val patchFile = File(ctx.filesDir, "wingbird/libapp.so")
    
    val patchPath = if (patchFile.exists() && patchFile.canRead() && patchFile.length() > 0) {
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

  override fun query(u: Uri, p: Array<String>?, s: String?, sa: Array<String>?, so: String?): Cursor? = null
  override fun getType(u: Uri): String? = null
  override fun insert(u: Uri, v: ContentValues?): Uri? = null
  override fun delete(u: Uri, s: String?, sa: Array<String>?): Int = 0
  override fun update(u: Uri, v: ContentValues?, s: String?, sa: Array<String>?): Int = 0
}
