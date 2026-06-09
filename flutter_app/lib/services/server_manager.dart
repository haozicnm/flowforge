/// Rust server lifecycle manager.
///
/// Starts the Rust backend as a child process, waits for it to be ready,
/// and stops it when the Flutter app exits.
library;

import 'dart:async';
import 'dart:io';

import 'flowforge_api.dart';

/// Manages the Rust backend server process.
class ServerManager {
  Process? _process;
  String _serverUrl = 'http://127.0.0.1:19529';
  bool _startedByUs = false;

  /// The server URL.
  String get serverUrl => _serverUrl;

  /// Start the Rust server.
  ///
  /// If [externalServerUrl] is provided, connects to an existing server
  /// instead of starting a new one (for development mode).
  Future<void> start({String? externalServerUrl}) async {
    if (externalServerUrl != null) {
      _serverUrl = externalServerUrl;
      _startedByUs = false;
    } else {
      await _startProcess();
      _startedByUs = true;
    }

    // Wait for server to be ready
    await _waitForReady();
  }

  /// Start the Rust server process.
  Future<void> _startProcess() async {
    // Find the workflow-engine binary
    final exePath = _findServerBinary();
    if (exePath == null) {
      throw ServerException('Cannot find flowforge server binary');
    }

    print('Starting server: $exePath');
    _process = await Process.start(
      exePath,
      [],
      environment: {
        'BIND': '127.0.0.1:19529',
        'RUST_LOG': 'flowforge=info',
      },
    );

    // Log server output
    _process!.stdout.transform(const SystemEncoding()).listen((line) {
      print('[server] $line');
    });
    _process!.stderr.transform(const SystemEncoding()).listen((line) {
      print('[server:err] $line');
    });
  }

  /// Find the server binary in common locations.
  String? _findServerBinary() {
    // Check environment variable
    final envPath = Platform.environment['FLOWFORGE_SERVER'];
    if (envPath != null && File(envPath).existsSync()) return envPath;

    // Check relative to the app
    final appDir = File(Platform.resolvedExecutable).parent.path;
    final candidates = [
      '$appDir/flowforge',
      '$appDir/../flowforge',
      '${Directory.current.path}/target/release/flowforge',
      '${Directory.current.path}/target/debug/flowforge',
    ];

    for (final path in candidates) {
      if (File(path).existsSync()) return path;
    }

    return null;
  }

  /// Wait for the server to be ready (poll /api/health).
  Future<void> _waitForReady() async {
    final api = FlowForgeApi(baseUrl: _serverUrl);
    final deadline = DateTime.now().add(const Duration(seconds: 15));

    while (DateTime.now().isBefore(deadline)) {
      try {
        final health = await api.health();
        if (health.status == 'ok') {
          print('Server ready: v${health.version}');
          api.dispose();
          return;
        }
      } catch (_) {
        // Server not ready yet, keep waiting
      }
      await Future.delayed(const Duration(milliseconds: 300));
    }

    api.dispose();
    throw ServerException('Server failed to start within 15 seconds');
  }

  /// Stop the server (if we started it).
  Future<void> stop() async {
    if (_startedByUs && _process != null) {
      print('Stopping server...');
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

/// Server exception.
class ServerException implements Exception {
  final String message;
  ServerException(this.message);

  @override
  String toString() => 'ServerException: $message';
}
