package com.flutter_rust_bridge.wingbird_sdk

import android.content.Context
import io.flutter.embedding.engine.plugins.FlutterPlugin
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import io.flutter.plugin.common.MethodChannel.MethodCallHandler
import io.flutter.plugin.common.MethodChannel.Result
import java.io.File
import java.util.zip.ZipFile

class WingbirdSdkPlugin: FlutterPlugin, MethodCallHandler {
    private lateinit var channel: MethodChannel
    private lateinit var context: Context

    override fun onAttachedToEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        context = binding.applicationContext
        channel = MethodChannel(binding.binaryMessenger, "com.wingbird.sdk/native")
        channel.setMethodCallHandler(this)
    }

    override fun onDetachedFromEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        channel.setMethodCallHandler(null)
    }

    override fun onMethodCall(call: MethodCall, result: Result) {
        if (call.method == "getLibAppPath") {
            val path = getLibAppPath()
            if (path != null) {
                result.success(path)
            } else {
                result.error("UNAVAILABLE", "Could not locate or extract libapp.so", null)
            }
        } else {
            result.notImplemented()
        }
    }

    private fun getLibAppPath(): String? {
        val libDir = context.applicationInfo.nativeLibraryDir
        val libAppFile = File(libDir, "libapp.so")

        if (libAppFile.exists()) {
            return libAppFile.absolutePath
        }

        return extractFromApk()
    }

    private fun extractFromApk(): String? {
        val apkFile = File(context.applicationInfo.sourceDir)
        val outputFile = File(context.cacheDir, "libapp.so")

        return try {
            ZipFile(apkFile).use { zip ->
                val entry = zip.entries().asSequence().firstOrNull { 
                    it.name.endsWith("libapp.so") 
                } ?: return null

                zip.getInputStream(entry).use { input ->
                    outputFile.outputStream().use { output ->
                        input.copyTo(output)
                    }
                }
            }
            outputFile.absolutePath
        } catch (e: Exception) {
            null
        }
    }
}
