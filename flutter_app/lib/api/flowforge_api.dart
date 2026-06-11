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
  final Map<String, dynamic>? configSchema;

  NodeTypeDef({
    required this.typeName,
    required this.displayName,
    required this.description,
    required this.category,
    required this.inputs,
    required this.outputs,
    this.configSchema,
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
      configSchema: json['config_schema'] as Map<String, dynamic>?,
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

/// A node in a workflow.
class WorkflowNode {
  final String id;
  final String type;
  String label;
  Map<String, dynamic> config;
  final Map<String, double> position;

  WorkflowNode({
    required this.id,
    required this.type,
    this.label = '',
    this.config = const {},
    Map<String, double>? position,
  }) : position = position ?? {'x': 0, 'y': 0};

  /// Convenience getters/setters for canvas positioning.
  double get positionX => position['x'] ?? 0;
  double get positionY => position['y'] ?? 0;
  set positionX(double v) => position['x'] = v;
  set positionY(double v) => position['y'] = v;

  factory WorkflowNode.fromJson(Map<String, dynamic> json) {
    return WorkflowNode(
      id: json['id'] as String,
      type: json['type'] as String,
      label: (json['label'] as String?) ?? '',
      config: (json['config'] as Map<String, dynamic>?) ?? {},
      position: {
        'x': (json['position']?['x'] as num?)?.toDouble() ?? 0,
        'y': (json['position']?['y'] as num?)?.toDouble() ?? 0,
      },
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'type': type,
        if (label.isNotEmpty) 'label': label,
        'config': config,
        'position': position,
      };
}

/// An edge connecting two nodes.
class WorkflowEdge {
  final String from;
  final String fromPort;
  final String to;
  final String toPort;

  WorkflowEdge({
    required this.from,
    this.fromPort = 'out',
    required this.to,
    this.toPort = 'in',
  });

  factory WorkflowEdge.fromJson(Map<String, dynamic> json) {
    return WorkflowEdge(
      from: json['from'] as String,
      fromPort: (json['from_port'] as String?) ?? 'out',
      to: json['to'] as String,
      toPort: (json['to_port'] as String?) ?? 'in',
    );
  }

  Map<String, dynamic> toJson() => {
        'from': from,
        'from_port': fromPort,
        'to': to,
        'to_port': toPort,
      };
}

/// Workflow model — matches Rust backend exactly.
class Workflow {
  final String id;
  final String name;
  final String description;
  final List<WorkflowNode> nodes;
  final List<WorkflowEdge> edges;
  final List<dynamic> variables;
  final DateTime createdAt;

  Workflow({
    required this.id,
    required this.name,
    this.description = '',
    this.nodes = const [],
    this.edges = const [],
    this.variables = const [],
    required this.createdAt,
  });

  factory Workflow.fromJson(Map<String, dynamic> json) {
    return Workflow(
      id: json['id'] as String,
      name: json['name'] as String,
      description: (json['description'] as String?) ?? '',
      nodes: (json['nodes'] as List?)
              ?.map((n) => WorkflowNode.fromJson(n as Map<String, dynamic>))
              .toList() ??
          [],
      edges: (json['edges'] as List?)
              ?.map((e) => WorkflowEdge.fromJson(e as Map<String, dynamic>))
              .toList() ??
          [],
      variables: (json['variables'] as List?) ?? [],
      createdAt: DateTime.parse(json['created_at'] as String),
    );
  }

  int get nodeCount => nodes.length;
  int get edgeCount => edges.length;
}

/// Execution result from the server.
class ExecutionResult {
  final String status;
  final Map<String, dynamic> nodeOutputs;
  final List<String> completed;
  final List<String> failed;
  final String? error;

  ExecutionResult({
    required this.status,
    this.nodeOutputs = const {},
    this.completed = const [],
    this.failed = const [],
    this.error,
  });

  factory ExecutionResult.fromJson(Map<String, dynamic> json) {
    return ExecutionResult(
      status: json['status'] as String,
      nodeOutputs: (json['node_outputs'] as Map<String, dynamic>?) ?? {},
      completed: (json['completed'] as List?)
              ?.map((e) => e as String)
              .toList() ??
          [],
      failed: (json['failed'] as List?)
              ?.map((e) => e as String)
              .toList() ??
          [],
      error: json['error'] as String?,
    );
  }

  bool get isSuccess => status == 'completed';
}

/// Result of a single-step execution.
class StepResult {
  final String status;
  final List<String> executed;
  final bool hasMore;
  final Map<String, dynamic> nodeOutputs;
  final List<String> completed;
  final List<String> failed;
  final String? error;

  StepResult({
    required this.status,
    this.executed = const [],
    this.hasMore = false,
    this.nodeOutputs = const {},
    this.completed = const [],
    this.failed = const [],
    this.error,
  });

  factory StepResult.fromJson(Map<String, dynamic> json) {
    return StepResult(
      status: json['status'] as String,
      executed: (json['executed'] as List?)?.map((e) => e as String).toList() ?? [],
      hasMore: json['has_more'] as bool? ?? false,
      nodeOutputs: (json['node_outputs'] as Map<String, dynamic>?) ?? {},
      completed: (json['completed'] as List?)?.map((e) => e as String).toList() ?? [],
      failed: (json['failed'] as List?)?.map((e) => e as String).toList() ?? [],
      error: json['error'] as String?,
    );
  }
}

/// FlowForge API client.
class FlowForgeApi {
  final String baseUrl;
  final http.Client _client;

  FlowForgeApi({required this.baseUrl}) : _client = http.Client();

  // ── Health ──

  Future<HealthResponse> health() async {
    final response = await _client.get(Uri.parse('$baseUrl/api/health'));
    if (response.statusCode != 200) {
      throw ApiException('Health check failed: ${response.statusCode}');
    }
    return HealthResponse.fromJson(
      jsonDecode(response.body) as Map<String, dynamic>,
    );
  }

  // ── Node Types ──

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

  // ── Workflow CRUD ──

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

  Future<Workflow> createWorkflow(String name, {String? description}) async {
    final response = await _client.post(
      Uri.parse('$baseUrl/api/workflows'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'name': name,
        if (description != null) 'description': description,
      }),
    );
    if (response.statusCode != 201) {
      throw ApiException('Failed to create workflow: ${response.statusCode}');
    }
    return Workflow.fromJson(
      jsonDecode(response.body) as Map<String, dynamic>,
    );
  }

  Future<Workflow> getWorkflow(String id) async {
    final response =
        await _client.get(Uri.parse('$baseUrl/api/workflows/$id'));
    if (response.statusCode != 200) {
      throw ApiException('Failed to get workflow: ${response.statusCode}');
    }
    return Workflow.fromJson(
      jsonDecode(response.body) as Map<String, dynamic>,
    );
  }

  Future<Workflow> updateWorkflow(
    String id, {
    String? name,
    String? description,
    List<WorkflowNode>? nodes,
    List<WorkflowEdge>? edges,
  }) async {
    final body = <String, dynamic>{};
    if (name != null) body['name'] = name;
    if (description != null) body['description'] = description;
    if (nodes != null) body['nodes'] = nodes.map((n) => n.toJson()).toList();
    if (edges != null) body['edges'] = edges.map((e) => e.toJson()).toList();

    final response = await _client.put(
      Uri.parse('$baseUrl/api/workflows/$id'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode(body),
    );
    if (response.statusCode != 200) {
      throw ApiException('Failed to update workflow: ${response.statusCode}');
    }
    return Workflow.fromJson(
      jsonDecode(response.body) as Map<String, dynamic>,
    );
  }

  Future<void> deleteWorkflow(String id) async {
    final response =
        await _client.delete(Uri.parse('$baseUrl/api/workflows/$id'));
    if (response.statusCode != 204) {
      throw ApiException('Failed to delete workflow: ${response.statusCode}');
    }
  }

  // ── Execution ──

  Future<ExecutionResult> executeWorkflow(String id) async {
    final response = await _client.post(
      Uri.parse('$baseUrl/api/workflows/$id/execute'),
      headers: {'Content-Type': 'application/json'},
    );
    if (response.statusCode != 200) {
      throw ApiException('Failed to execute: ${response.statusCode}');
    }
    return ExecutionResult.fromJson(
      jsonDecode(response.body) as Map<String, dynamic>,
    );
  }

  /// Single-step execution: run the next topological level.
  Future<StepResult> executeStep(String id, {
    required List<String> completed,
    required List<String> failed,
    required Map<String, dynamic> nodeOutputs,
  }) async {
    final response = await _client.post(
      Uri.parse('$baseUrl/api/workflows/$id/execute-step'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'completed': completed,
        'failed': failed,
        'node_outputs': nodeOutputs,
      }),
    );
    if (response.statusCode != 200) {
      throw ApiException('Failed to step: ${response.statusCode}');
    }
    return StepResult.fromJson(
      jsonDecode(response.body) as Map<String, dynamic>,
    );
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
