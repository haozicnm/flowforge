// Rust server lifecycle manager — auto-spawn + connect + cleanup.

import 'dart:async';
import 'dart:io' show File, Platform, Process, ProcessSignal, exit;
import 'package:flutter/foundation.dart';

import '../api/flowforge_api.dart';

class ServerManager {
  String _serverUrl = 'http://127.0.0.1:19529';
  bool _connected = false;
  Process? _backendProcess;

  String get serverUrl => _serverUrl;
  bool get isConnected => _connected;

  /// Start (spawn + connect) the backend.
  Future<void> start({String? externalServerUrl}) async {
    if (externalServerUrl != null && externalServerUrl.isNotEmpty) {
      _serverUrl = externalServerUrl;
      await _waitForReady();
      return;
    }

    _spawnBackend();
    _registerCleanup();
    await _waitForReady();
  }

  void _spawnBackend() {
    final binary = _findBackend();
    if (binary == null) {
      debugPrint('No bundled backend. Expecting server at $_serverUrl');
      return;
    }

    debugPrint('Launching backend: $binary');
    try {
      _backendProcess = Process.startSync(binary, [],
        mode: ProcessStartMode.detachedWithStdio,
      );
      debugPrint('Backend PID: ${_backendProcess!.pid}');
    } catch (e) {
      debugPrint('Failed to launch backend: $e');
    }
  }

  String? _findBackend() {
    final exeName = Platform.isWindows ? 'backend/flowforge.exe' : 'backend/flowforge';
    final altName = Platform.isWindows ? r'backend\flowforge.exe' : 'backend/flowforge';

    // Try current directory
    if (File(exeName).existsSync()) return exeName;
    if (File(altName).existsSync()) return altName;

    // Try relative to the executable
    try {
      final execDir = File(Platform.resolvedExecutable).parent.path;
      final p = '$execDir/$exeName';
      if (File(p).existsSync()) return p;
    } catch (_) {}

    return null;
  }

  Future<void> _waitForReady() async {
    final api = FlowForgeApi(baseUrl: _serverUrl);
    final deadline = DateTime.now().add(const Duration(seconds: 15));

    while (DateTime.now().isBefore(deadline)) {
      try {
        final health = await api.health();
        if (health.status == 'ok') {
          debugPrint('Backend v${health.version} ready at $_serverUrl');
          _connected = true;
          api.dispose();
          return;
        }
      } catch (_) {}
      await Future.delayed(const Duration(milliseconds: 500));
    }

    api.dispose();
    debugPrint('Backend not reachable at $_serverUrl');
  }

  void _registerCleanup() {
    ProcessSignal.sigint.watch().listen((_) => _kill());
    ProcessSignal.sigterm.watch().listen((_) => _kill());
  }

  void _kill() {
    _backendProcess?.kill();
    exit(0);
  }

  void stop() => _kill();
}

class ServerException implements Exception {
  final String message;
  ServerException(this.message);
  @override
  String toString() => 'ServerException: $message';
}
