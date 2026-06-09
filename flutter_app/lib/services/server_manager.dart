/// Rust server lifecycle manager.
library;

import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:flutter/foundation.dart';

import '../api/flowforge_api.dart';

/// Manages the Rust backend server process.
class ServerManager {
  Process? _process;
  String _serverUrl = 'http://127.0.0.1:19529';
  bool _startedByUs = false;

  String get serverUrl => _serverUrl;

  Future<void> start({String? externalServerUrl}) async {
    if (externalServerUrl != null) {
      _serverUrl = externalServerUrl;
      _startedByUs = false;
    } else {
      await _startProcess();
      _startedByUs = true;
    }
    await _waitForReady();
  }

  Future<void> _startProcess() async {
    final exePath = _findServerBinary();
    if (exePath == null) {
      throw ServerException('Cannot find flowforge server binary');
    }

    debugPrint('Starting server: $exePath');
    _process = await Process.start(
      exePath,
      [],
      environment: {
        'BIND': '127.0.0.1:19529',
        'RUST_LOG': 'flowforge=info',
      },
    );

    _process!.stdout.transform(utf8.decoder).listen((line) {
      debugPrint('[server] $line');
    });
    _process!.stderr.transform(utf8.decoder).listen((line) {
      debugPrint('[server:err] $line');
    });
  }

  String? _findServerBinary() {
    final envPath = Platform.environment['FLOWFORGE_SERVER'];
    if (envPath != null && File(envPath).existsSync()) return envPath;

    final appDir = File(Platform.resolvedExecutable).parent.path;
    final candidates = [
      '$appDir/flowforge',
      '$appDir/flowforge.exe',
      '$appDir/../flowforge',
      '${Directory.current.path}/target/release/flowforge',
      '${Directory.current.path}/target/release/flowforge.exe',
      '${Directory.current.path}/target/debug/flowforge',
      '${Directory.current.path}/target/debug/flowforge.exe',
    ];

    for (final path in candidates) {
      if (File(path).existsSync()) return path;
    }
    return null;
  }

  Future<void> _waitForReady() async {
    final api = FlowForgeApi(baseUrl: _serverUrl);
    final deadline = DateTime.now().add(const Duration(seconds: 15));

    while (DateTime.now().isBefore(deadline)) {
      try {
        final health = await api.health();
        if (health.status == 'ok') {
          debugPrint('Server ready: v${health.version}');
          api.dispose();
          return;
        }
      } catch (_) {}
      await Future.delayed(const Duration(milliseconds: 300));
    }

    api.dispose();
    throw ServerException('Server failed to start within 15 seconds');
  }

  Future<void> stop() async {
    if (_startedByUs && _process != null) {
      debugPrint('Stopping server...');
      _process!.kill(ProcessSignal.sigterm);
      await _process!.exitCode.timeout(
        const Duration(seconds: 3),
        onTimeout: () {
          _process!.kill(ProcessSignal.sigkill);
          return -1;
        },
      );
      _process = null;
    }
  }
}

class ServerException implements Exception {
  final String message;
  ServerException(this.message);

  @override
  String toString() => 'ServerException: $message';
}
