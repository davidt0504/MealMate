package dev.mealmate.temp

import android.content.ActivityNotFoundException
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

/**
 * OPT-001's platform side: text shared to Kimatta and opening a link in the browser, over
 * one method channel (`lib/app/share_channel.dart`). No plugin, no other behaviour.
 */
class MainActivity : FlutterActivity() {
    private var channel: MethodChannel? = null

    /** The cold-start share, handed to Dart once by `takeSharedText`. */
    private var pendingShare: String? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Only a fresh launch carries a new share: a recreated activity or a relaunch from
        // Recents still holds the old intent, and importing it again would surprise the user.
        val fromHistory = intent.flags and Intent.FLAG_ACTIVITY_LAUNCHED_FROM_HISTORY != 0
        if (savedInstanceState == null && !fromHistory) {
            pendingShare = sharedText(intent)
        }
        intent.removeExtra(Intent.EXTRA_TEXT)
    }

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        channel = MethodChannel(flutterEngine.dartExecutor.binaryMessenger, "kimatta/share").apply {
            setMethodCallHandler { call, result ->
                when (call.method) {
                    "takeSharedText" -> {
                        result.success(pendingShare)
                        pendingShare = null
                    }
                    "openUrl" -> {
                        openUrl(call.arguments as? String)
                        result.success(null)
                    }
                    else -> result.notImplemented()
                }
            }
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        sharedText(intent)?.let { channel?.invokeMethod("sharedText", it) }
        intent.removeExtra(Intent.EXTRA_TEXT)
    }

    private fun sharedText(intent: Intent?): String? =
        if (intent?.action == Intent.ACTION_SEND && intent.type == "text/plain") {
            intent.getStringExtra(Intent.EXTRA_TEXT)
        } else {
            null
        }

    /** Only http(s); with no browser installed the tap does nothing. */
    private fun openUrl(url: String?) {
        val uri = url?.let(Uri::parse) ?: return
        if (uri.scheme != "http" && uri.scheme != "https") return
        try {
            startActivity(Intent(Intent.ACTION_VIEW, uri).addCategory(Intent.CATEGORY_BROWSABLE))
        } catch (_: ActivityNotFoundException) {
        }
    }
}
