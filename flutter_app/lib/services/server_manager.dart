// Rust server lifecycle manager — connect-only mode.
// The Rust backend is started separately (manually or by script).
library;

import 'dart:async';
import 'package:flutter/foundation.dart';

import '../api/flowforge_api.dart';

/// Connects to an already-running FlowForge backend.
/// Does NOT spawn any process — the backend must be started separately.
class ServerManager {
  String _serverUrl = 'http://127.0.0.1:19529';
  bool _connected = false;

  String get serverUrl => _serverUrl;
  bool get isConnected => _connected;

  /// Connect to the backend at [url], or default 127.0.0.1:19529.
  Future<void> start({String? externalServerUrl}) async {
    if (externalServerUrl != null && externalServerUrl.isNotEmpty) {
      _serverUrl = externalServerUrl;
    }
    await _waitForReady();
  }

  Future<void> _waitForReady() async {
    final api = FlowForgeApi(baseUrl: _serverUrl);
    final deadline = DateTime.now().add(const Duration(seconds: 10));

    while (DateTime.now().isBefore(deadline)) {
      try {
        final health = await api.health();
        if (health.status == 'ok') {
          debugPrint('Connected to FlowForge backend v${health.version} at $_serverUrl');
          _connected = true;
          api.dispose();
          return;
        }
      } catch (_) {}
      await Future.delayed(const Duration(milliseconds: 500));
    }

    api.dispose();
    debugPrint('WARNING: Backend not reachable at $_serverUrl — UI will show errors');
    // Don't throw — let the app start and show connection errors gracefully
  }

  /// No-op — we don't own the server process.
  void stop() {}
}

class ServerException implements Exception {
  final String message;
  ServerException(this.message);

  @override
  String toString() => 'ServerException: $message';
}
