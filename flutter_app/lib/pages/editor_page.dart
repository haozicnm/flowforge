// Editor page — visual canvas + form editing + execution.
import 'dart:convert';
import 'package:flutter/material.dart';
import '../api/flowforge_api.dart';
import '../theme/flowforge_theme.dart';
import '../widgets/ff_widgets.dart';
import '../widgets/canvas_editor.dart';
import '../widgets/code_editor.dart';

class EditorPage extends StatefulWidget {
  final FlowForgeApi api;
  final String? workflowId;

  const EditorPage({super.key, required this.api, this.workflowId});

  @override
  State<EditorPage> createState() => _EditorPageState();
}

class _EditorPageState extends State<EditorPage> {
  Workflow? _workflow;
  bool _loading = false;
  bool _isExecuting = false;
  bool _isSaving = false;
  String _output = '';
  String? _error;
  final _nameController = TextEditingController();

  List<WorkflowNode> _nodes = [];
  List<WorkflowEdge> _edges = [];
  List<NodeTypeDef> _nodeTypes = [];

  // View mode: canvas, form, or code
  int _viewMode = 0; // 0=canvas, 1=form, 2=code
  String? _selectedNodeId;
  String _codeJson = '';

  @override
  void initState() {
    super.initState();
    _loadNodeTypes();
    if (widget.workflowId != null) {
      _loadWorkflow(widget.workflowId!);
    }
  }

  @override
  void didUpdateWidget(EditorPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.workflowId != oldWidget.workflowId && widget.workflowId != null) {
      _loadWorkflow(widget.workflowId!);
    }
  }

  Future<void> _loadNodeTypes() async {
    try {
      final types = await widget.api.nodeTypes();
      setState(() => _nodeTypes = types);
    } catch (_) {}
  }

  Future<void> _loadWorkflow(String id) async {
    setState(() { _loading = true; _error = null; });
    try {
      final wf = await widget.api.getWorkflow(id);
      setState(() {
        _workflow = wf;
        _nodes = List.from(wf.nodes);
        _edges = List.from(wf.edges);
        _nameController.text = wf.name;
        _loading = false;
        _selectedNodeId = null;
      });
    } catch (e) {
      setState(() { _error = e.toString(); _loading = false; });
    }
  }

  Future<void> _save() async {
    if (_workflow == null) return;
    setState(() => _isSaving = true);
    try {
      final updated = await widget.api.updateWorkflow(
        _workflow!.id,
        name: _nameController.text,
        nodes: _nodes,
        edges: _edges,
      );
      setState(() { _workflow = updated; _isSaving = false; });
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('已保存'), duration: Duration(seconds: 1)),
        );
      }
    } catch (e) {
      setState(() => _isSaving = false);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('保存失败: $e')),
        );
      }
    }
  }

  Future<void> _execute() async {
    if (_workflow == null) return;
    setState(() { _isExecuting = true; _output = '执行中...'; });
    try {
      await _save();
      final result = await widget.api.executeWorkflow(_workflow!.id);
      final buf = StringBuffer();
      if (result.isSuccess) {
        buf.writeln('✅ 执行完成');
        buf.writeln();
        buf.writeln('节点执行顺序:');
        for (final nodeId in result.completed) {
          buf.writeln('  ✓ $nodeId');
        }
        if (result.nodeOutputs.isNotEmpty) {
          buf.writeln();
          buf.writeln('节点输出:');
          result.nodeOutputs.forEach((nodeId, outputs) {
            if (outputs is Map && outputs.isNotEmpty) {
              buf.writeln('  $nodeId:');
              outputs.forEach((port, value) {
                buf.writeln('    $port: ${const JsonEncoder.withIndent("  ").convert(value)}');
              });
            }
          });
        }
      } else {
        buf.writeln('❌ 执行失败');
        buf.writeln(result.error ?? '未知错误');
      }
      setState(() { _output = buf.toString(); _isExecuting = false; });
    } catch (e) {
      setState(() { _output = '❌ 执行出错: $e'; _isExecuting = false; });
    }
  }

  void _addNode() {
    final idController = TextEditingController();
    String selectedType = _nodeTypes.isNotEmpty ? _nodeTypes.first.typeName : 'log';

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('添加节点'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: idController,
              decoration: const InputDecoration(labelText: '节点 ID', border: OutlineInputBorder()),
            ),
            const SizedBox(height: 12),
            DropdownButtonFormField<String>(
              value: selectedType,
              decoration: const InputDecoration(labelText: '类型', border: OutlineInputBorder()),
              items: _nodeTypes.map((t) => DropdownMenuItem(
                value: t.typeName,
                child: Text(t.displayName),
              )).toList(),
              onChanged: (v) => selectedType = v ?? selectedType,
            ),
          ],
        ),
        actions: [
          TextButton(onPressed: () => Navigator.pop(ctx), child: const Text('取消')),
          ElevatedButton(
            onPressed: () {
              if (idController.text.isNotEmpty) {
                setState(() {
                  _nodes.add(WorkflowNode(
                    id: idController.text,
                    type: selectedType,
                    config: selectedType == 'log' ? {'level': 'info', 'message': ''} : {},
                    position: {'x': 100 + _nodes.length * 220.0, 'y': 100},
                  ));
                });
                Navigator.pop(ctx);
              }
            },
            child: const Text('添加'),
          ),
        ],
      ),
    );
  }

  void _removeNode(String nodeId) {
    setState(() {
      _nodes.removeWhere((n) => n.id == nodeId);
      _edges.removeWhere((e) => e.from == nodeId || e.to == nodeId);
      if (_selectedNodeId == nodeId) _selectedNodeId = null;
    });
  }

  void _addEdge() {
    if (_nodes.length < 2) return;
    String fromNode = _nodes[0].id;
    String toNode = _nodes.length > 1 ? _nodes[1].id : _nodes[0].id;

    showDialog(
      context: context,
      builder: (ctx) => StatefulBuilder(
        builder: (ctx, setDialogState) => AlertDialog(
          title: const Text('添加连接'),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              DropdownButtonFormField<String>(
                value: fromNode,
                decoration: const InputDecoration(labelText: '从', border: OutlineInputBorder()),
                items: _nodes.map((n) => DropdownMenuItem(value: n.id, child: Text(n.label.isNotEmpty ? n.label : n.id))).toList(),
                onChanged: (v) => setDialogState(() => fromNode = v ?? fromNode),
              ),
              const SizedBox(height: 12),
              DropdownButtonFormField<String>(
                value: toNode,
                decoration: const InputDecoration(labelText: '到', border: OutlineInputBorder()),
                items: _nodes.map((n) => DropdownMenuItem(value: n.id, child: Text(n.label.isNotEmpty ? n.label : n.id))).toList(),
                onChanged: (v) => setDialogState(() => toNode = v ?? toNode),
              ),
            ],
          ),
          actions: [
            TextButton(onPressed: () => Navigator.pop(ctx), child: const Text('取消')),
            ElevatedButton(
              onPressed: () {
                setState(() {
                  _edges.add(WorkflowEdge(from: fromNode, to: toNode));
                });
                Navigator.pop(ctx);
              },
              child: const Text('添加'),
            ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final ext = theme.extension<FlowForgeThemeExtension>()!;

    if (widget.workflowId == null) {
      return _buildNoSelection(theme, ext);
    }
    if (_loading) return const Center(child: CircularProgressIndicator());
    if (_error != null) return _buildError(theme);

    return Padding(
      padding: const EdgeInsets.all(FlowForgeSpacing.lg),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildTopBar(theme, ext),
          const SizedBox(height: FlowForgeSpacing.md),
          Expanded(
            child: Row(
              children: [
                // Canvas / Form / Code editor
                Expanded(
                  flex: 3,
                  child: _viewMode == 0
                      ? _buildCanvas(ext)
                      : _viewMode == 1
                          ? _buildFormEditor(theme, ext)
                          : _buildCodeEditor(theme, ext),
                ),
                const SizedBox(width: FlowForgeSpacing.md),
                // Right panel: properties + output
                Expanded(
                  flex: 2,
                  child: Column(
                    children: [
                      if (_selectedNodeId != null) ...[
                        Expanded(child: _buildPropertiesPanel(theme, ext)),
                        const SizedBox(height: FlowForgeSpacing.md),
                      ],
                      Expanded(child: _buildOutputPanel(theme, ext)),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildCanvas(FlowForgeThemeExtension ext) {
    return Container(
      decoration: BoxDecoration(
        color: ext.surfaceColor,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: ext.borderColor),
      ),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(8),
        child: CanvasEditor(
          nodes: _nodes,
          edges: _edges,
          nodeTypes: _nodeTypes,
          selectedNodeId: _selectedNodeId,
          onNodeSelected: (id) => setState(() => _selectedNodeId = id),
          onChanged: (nodes, edges) => setState(() {}),
        ),
      ),
    );
  }

  Widget _buildCodeEditor(ThemeData theme, FlowForgeThemeExtension ext) {
    return Container(
      decoration: BoxDecoration(
        color: ext.surfaceColor,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: ext.borderColor),
      ),
      child: Column(
        children: [
          // Toolbar
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            child: Row(
              children: [
                FfText('JSON 编辑器', fontSize: 12, fontWeight: FontWeight.w600),
                const Spacer(),
                FfButton(
                  onTap: () {
                    // Format JSON
                    setState(() => _codeJson = formatJson(_codeJson));
                  },
                  builder: (ctx, _) => const Icon(Icons.format_align_left, size: 16),
                ),
                const SizedBox(width: 4),
                FfButton(
                  onTap: () {
                    // Apply changes from code to nodes/edges
                    _applyCodeChanges();
                  },
                  builder: (ctx, _) => Icon(Icons.check, size: 16, color: Colors.green),
                ),
              ],
            ),
          ),
          const FfDivider(),
          // Editor
          Expanded(
            child: Padding(
              padding: const EdgeInsets.all(4),
              child: CodeEditor(
                initialCode: _codeJson.isEmpty
                    ? formatJson(jsonEncode({
                        'name': _nameController.text,
                        'nodes': _nodes.map((n) => n.toJson()).toList(),
                        'edges': _edges.map((e) => e.toJson()).toList(),
                      }))
                    : _codeJson,
                onChanged: (v) => _codeJson = v,
              ),
            ),
          ),
        ],
      ),
    );
  }

  void _applyCodeChanges() {
    try {
      final obj = jsonDecode(_codeJson);
      if (obj is Map<String, dynamic>) {
        setState(() {
          if (obj['name'] != null) _nameController.text = obj['name'] as String;
          if (obj['nodes'] != null) {
            _nodes = (obj['nodes'] as List)
                .map((n) => WorkflowNode.fromJson(n as Map<String, dynamic>))
                .toList();
          }
          if (obj['edges'] != null) {
            _edges = (obj['edges'] as List)
                .map((e) => WorkflowEdge.fromJson(e as Map<String, dynamic>))
                .toList();
          }
        });
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(content: Text('代码已应用'), duration: Duration(seconds: 1)),
          );
        }
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('JSON 解析失败: $e')),
        );
      }
    }
  }

  Widget _buildFormEditor(ThemeData theme, FlowForgeThemeExtension ext) {
    return Container(
      decoration: BoxDecoration(
        color: ext.surfaceColor,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: ext.borderColor),
      ),
      child: Column(
        children: [
          // Nodes header
          Padding(
            padding: const EdgeInsets.all(FlowForgeSpacing.md),
            child: Row(
              children: [
                FfText('节点 (${_nodes.length})', fontSize: 12, fontWeight: FontWeight.w600),
                const Spacer(),
                FfButton(
                  onTap: _addNode,
                  builder: (ctx, _) => const Icon(Icons.add, size: 16),
                ),
                const SizedBox(width: 4),
                FfButton(
                  onTap: _addEdge,
                  builder: (ctx, _) => const Icon(Icons.add_link, size: 16),
                ),
              ],
            ),
          ),
          const FfDivider(),
          // Node list
          Expanded(
            child: _nodes.isEmpty
                ? Center(child: FfText('暂无节点', fontSize: 13, color: theme.colorScheme.onSurface.withValues(alpha: 0.4)))
                : ListView.builder(
                    itemCount: _nodes.length,
                    itemBuilder: (ctx, i) => _buildNodeTile(_nodes[i], i, theme, ext),
                  ),
          ),
          // Edges
          if (_edges.isNotEmpty) ...[
            const FfDivider(),
            Padding(
              padding: const EdgeInsets.all(FlowForgeSpacing.md),
              child: FfText('连接 (${_edges.length})', fontSize: 12, fontWeight: FontWeight.w600),
            ),
            ..._edges.map((e) => Padding(
              padding: const EdgeInsets.symmetric(horizontal: FlowForgeSpacing.md, vertical: 2),
              child: Row(
                children: [
                  Expanded(child: FfText('${e.from} → ${e.to}', fontSize: 12)),
                  GestureDetector(
                    onTap: () => setState(() => _edges.remove(e)),
                    child: Icon(Icons.close, size: 14, color: theme.colorScheme.onSurface.withValues(alpha: 0.4)),
                  ),
                ],
              ),
            )),
          ],
        ],
      ),
    );
  }

  Widget _buildNodeTile(WorkflowNode node, int index, ThemeData theme, FlowForgeThemeExtension ext) {
    return ListTile(
      dense: true,
      leading: Icon(_nodeIcon(node.type), size: 18, color: _nodeColor(node.type)),
      title: FfText(node.label.isNotEmpty ? node.label : node.id, fontSize: 13),
      subtitle: FfText(node.type, fontSize: 11, color: theme.colorScheme.onSurface.withValues(alpha: 0.5)),
      trailing: GestureDetector(
        onTap: () => _removeNode(node.id),
        child: Icon(Icons.delete_outline, size: 16, color: theme.colorScheme.onSurface.withValues(alpha: 0.4)),
      ),
      selected: _selectedNodeId == node.id,
      selectedTileColor: ext.brandColor.withValues(alpha: 0.08),
      onTap: () => setState(() => _selectedNodeId = node.id),
    );
  }

  Widget _buildPropertiesPanel(ThemeData theme, FlowForgeThemeExtension ext) {
    final node = _nodes.where((n) => n.id == _selectedNodeId).firstOrNull;
    if (node == null) return const SizedBox.shrink();

    return Container(
      decoration: BoxDecoration(
        color: ext.surfaceColor,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: ext.borderColor),
      ),
      padding: const EdgeInsets.all(FlowForgeSpacing.md),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(_nodeIcon(node.type), size: 18, color: _nodeColor(node.type)),
              const SizedBox(width: 8),
              Expanded(child: FfText(node.id, fontSize: 14, fontWeight: FontWeight.w600)),
              GestureDetector(
                onTap: () => _removeNode(node.id),
                child: Icon(Icons.delete_outline, size: 16, color: Colors.red.withValues(alpha: 0.7)),
              ),
            ],
          ),
          const SizedBox(height: 12),
          // Label field
          TextField(
            controller: TextEditingController(text: node.label),
            decoration: const InputDecoration(labelText: '标签', border: OutlineInputBorder(), isDense: true),
            onChanged: (v) => node.label = v,  // WorkflowNode.label needs to be non-final
          ),
          const SizedBox(height: 8),
          // Config editor
          FfText('配置 (JSON)', fontSize: 12, fontWeight: FontWeight.w600),
          const SizedBox(height: 4),
          Expanded(
            child: Container(
              decoration: BoxDecoration(
                color: theme.colorScheme.surface,
                borderRadius: BorderRadius.circular(4),
                border: Border.all(color: ext.borderColor),
              ),
              padding: const EdgeInsets.all(8),
              child: SingleChildScrollView(
                child: Text(
                  const JsonEncoder.withIndent('  ').convert(node.config),
                  style: TextStyle(fontSize: 11, fontFamily: 'monospace', color: theme.colorScheme.onSurface),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildTopBar(ThemeData theme, FlowForgeThemeExtension ext) {
    return Row(
      children: [
        // Workflow name
        Expanded(
          flex: 2,
          child: TextField(
            controller: _nameController,
            style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w600),
            decoration: const InputDecoration(
              border: InputBorder.none,
              contentPadding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            ),
          ),
        ),
        // View toggle
        SegmentedButton<int>(
          segments: const [
            ButtonSegment(value: 0, icon: Icon(Icons.dashboard, size: 16), label: Text('画布')),
            ButtonSegment(value: 1, icon: Icon(Icons.list, size: 16), label: Text('列表')),
            ButtonSegment(value: 2, icon: Icon(Icons.code, size: 16), label: Text('代码')),
          ],
          selected: {_viewMode},
          onSelectionChanged: (v) {
            setState(() {
              _viewMode = v.first;
              if (_viewMode == 2) {
                // Sync code view from nodes/edges
                _codeJson = formatJson(jsonEncode({
                  'name': _nameController.text,
                  'nodes': _nodes.map((n) => n.toJson()).toList(),
                  'edges': _edges.map((e) => e.toJson()).toList(),
                }));
              }
            });
          },
          style: ButtonStyle(visualDensity: VisualDensity.compact),
        ),
        const SizedBox(width: FlowForgeSpacing.md),
        // Actions
        FfButton(
          onTap: _isSaving ? null : _save,
          builder: (ctx, hovering) => Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                if (_isSaving)
                  const SizedBox(width: 14, height: 14, child: CircularProgressIndicator(strokeWidth: 2))
                else
                  const Icon(Icons.save, size: 16),
                const SizedBox(width: 4),
                FfText('保存', fontSize: 12),
              ],
            ),
          ),
        ),
        const SizedBox(width: 8),
        FfButton(
          onTap: _isExecuting ? null : _execute,
          builder: (ctx, hovering) => Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                if (_isExecuting)
                  const SizedBox(width: 14, height: 14, child: CircularProgressIndicator(strokeWidth: 2))
                else
                  Icon(Icons.play_arrow, size: 16, color: Colors.green),
                const SizedBox(width: 4),
                FfText('执行', fontSize: 12),
              ],
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildOutputPanel(ThemeData theme, FlowForgeThemeExtension ext) {
    return Container(
      decoration: BoxDecoration(
        color: ext.surfaceColor,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: ext.borderColor),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.all(FlowForgeSpacing.md),
            child: FfText('执行输出', fontSize: 12, fontWeight: FontWeight.w600),
          ),
          const FfDivider(),
          Expanded(
            child: Padding(
              padding: const EdgeInsets.all(FlowForgeSpacing.md),
              child: SelectableText(
                _output.isEmpty ? '尚未执行' : _output,
                style: TextStyle(
                  fontSize: 12,
                  fontFamily: 'monospace',
                  color: _output.startsWith('❌')
                      ? Colors.red
                      : theme.colorScheme.onSurface,
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildNoSelection(ThemeData theme, FlowForgeThemeExtension ext) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.edit_outlined, size: 64, color: ext.brandColor.withValues(alpha: 0.3)),
          const SizedBox(height: FlowForgeSpacing.md),
          FfText('选择一个工作流开始编辑', fontSize: 16,
            color: theme.colorScheme.onSurface.withValues(alpha: 0.5)),
        ],
      ),
    );
  }

  Widget _buildError(ThemeData theme) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(Icons.error_outline, size: 48, color: Colors.red),
          const SizedBox(height: 12),
          Text(_error!, style: const TextStyle(color: Colors.red)),
          const SizedBox(height: 12),
          ElevatedButton(
            onPressed: widget.workflowId != null ? () => _loadWorkflow(widget.workflowId!) : null,
            child: const Text('重试'),
          ),
        ],
      ),
    );
  }

  IconData _nodeIcon(String type) {
    switch (type) {
      case 'log': return Icons.text_snippet;
      case 'http': return Icons.http;
      case 'delay': return Icons.timer;
      case 'shell': return Icons.terminal;
      case 'script': return Icons.code;
      case 'webhook': return Icons.webhook;
      default: return Icons.circle;
    }
  }

  Color _nodeColor(String type) {
    switch (type) {
      case 'log': return Colors.teal;
      case 'http': return Colors.blue;
      case 'delay': return Colors.orange;
      case 'shell': return Colors.red;
      case 'script': return Colors.purple;
      case 'webhook': return Colors.green;
      default: return Colors.grey;
    }
  }
}
