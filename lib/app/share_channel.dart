import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

/// The Android side of OPT-001: text shared to Kimatta (`ACTION_SEND`) and opening a link in
/// the browser. A small method channel in `MainActivity` rather than a plugin; every call
/// tolerates a platform without it (tests, iOS later) by doing nothing.
class ShareChannel {
  const ShareChannel();

  static const _channel = MethodChannel('kimatta/share');

  /// The text the app was cold-started with by a share, once; `null` afterwards.
  Future<String?> takeSharedText() async {
    try {
      return await _channel.invokeMethod<String>('takeSharedText');
    } on MissingPluginException {
      return null;
    } on PlatformException {
      return null;
    }
  }

  /// Shares that arrive while the app is running.
  void listen(void Function(String text) onShared) {
    _channel.setMethodCallHandler((call) async {
      if (call.method == 'sharedText' && call.arguments is String) {
        onShared(call.arguments as String);
      }
    });
  }

  /// Opens an http(s) link in the browser; anything else, or no browser, does nothing.
  Future<void> openUrl(String url) async {
    try {
      await _channel.invokeMethod<void>('openUrl', url);
    } on MissingPluginException {
      return;
    } on PlatformException {
      return;
    }
  }
}

final shareChannelProvider = Provider<ShareChannel>(
  (_) => const ShareChannel(),
);

final _url = RegExp(r'https?://\S+', caseSensitive: false);

/// The first http(s) link in shared text, without the sentence punctuation that commonly
/// trails it ("Try this: https://…/recipe."). `null` when the text holds no link.
String? firstUrl(String text) {
  final match = _url.firstMatch(text)?.group(0);
  return match?.replaceFirst(RegExp(r'''[.,;:!?)\]}'"»”]+$'''), '');
}
