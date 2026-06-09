/// FlowForge API client — talks to the Rust backend via HTTP.
///
/// This is the ONLY layer that communicates with the backend.
/// All UI code goes through this client.
library;

import 'dart:convert';
import 'package:http/http.dart' as http;

/// Health check response from the server.
class HealthResponse {
  final String version;
  final String status;

  HealthResponse({required this.version, required this.status});

  factory HealthResponse.fromJson(Map<String, dynamic> json) {
    return HealthResponse(
      version: json['version'] as String,
      status: json['status'] as String,
    );
  }
}

/// Node type definition from the server.
class NodeTypeDef {
  final String typeName;
  final String displayName;
  final String description;
  final String category;
  final List<PortDef> inputs;
  final List<PortDef> outputs;

  NodeTypeDef({
    required this.typeName,
    required this.displayName,
    required this.description,
    required this.category,
    required this.inputs,
    required this.outputs,
  });

  factory NodeTypeDef.fromJson(Map<String, dynamic> json) {
    return NodeTypeDef(
      typeName: json['type_name'] as String,
      displayName: json['display_name'] as String,
      description: json['description'] as String,
      category: json['category'] as String,
      inputs: (json['inputs'] as List)
          .map((p) => PortDef.fromJson(p as Map<String, dynamic>))
          .toList(),
      outputs: (json['outputs'] as List)
          .map((p) => PortDef.fromJson(p as Map<String, dynamic>))
          .toList(),
    );
  }
}

/// Port definition (input or output).
class PortDef {
  final String label;
  final String dataType;
  final bool required;

  PortDef({
    required this.label,
    this.dataType = 'any',
    this.required = false,
  });

  factory PortDef.fromJson(Map<String, dynamic> json) {
    return PortDef(
      label: json['label'] as String,
      dataType: (json['data_type'] as String?) ?? 'any',
      required: (json['required'] as bool?) ?? false,
    );
  }
}

/// Workflow model.
class Workflow {
  final String id;
  final String name;
  final String? description;
  final String? yaml;
  final DateTime createdAt;

  Workflow({
    required this.id,
    required this.name,
    this.description,
    this.yaml,
    required this.createdAt,
  });

  factory Workflow.fromJson(Map<String, dynamic> json) {
    return Workflow(
      id: json['id'] as String,
      name: json['name'] as String,
      description: json['description'] as String?,
      yaml: json['yaml'] as String?,
      createdAt: DateTime.parse(json['created_at'] as String),
    );
  }
}

/// FlowForge API client.
class FlowForgeApi {
  final String baseUrl;
  final http.Client _client;

  FlowForgeApi({required this.baseUrl}) : _client = http.Client();

  /// Check if the server is healthy.
  Future<HealthResponse> health() async {
    final response = await _client.get(Uri.parse('$baseUrl/api/health'));
    if (response.statusCode != 200) {
      throw ApiException('Health check failed: ${response.statusCode}');
    }
    return HealthResponse.fromJson(
      jsonDecode(response.body) as Map<String, dynamic>,
    );
  }

  /// Get all registered node types.
  Future<List<NodeTypeDef>> nodeTypes() async {
    final response =
        await _client.get(Uri.parse('$baseUrl/api/nodes/types'));
    if (response.statusCode != 200) {
      throw ApiException('Failed to get node types: ${response.statusCode}');
    }
    final list = jsonDecode(response.body) as List;
    return list
        .map((j) => NodeTypeDef.fromJson(j as Map<String, dynamic>))
        .toList();
  }

  /// List all workflows.
  Future<List<Workflow>> listWorkflows() async {
    final response =
        await _client.get(Uri.parse('$baseUrl/api/workflows'));
    if (response.statusCode != 200) {
      throw ApiException(
          'Failed to list workflows: ${response.statusCode}');
    }
    final list = jsonDecode(response.body) as List;
    return list
        .map((j) => Workflow.fromJson(j as Map<String, dynamic>))
        .toList();
  }

  /// Dispose the HTTP client.
  void dispose() {
    _client.close();
  }
}

/// API exception.
class ApiException implements Exception {
  final String message;
  ApiException(this.message);

  @override
  String toString() => 'ApiException: $message';
}
