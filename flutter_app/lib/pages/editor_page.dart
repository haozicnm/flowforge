// Editor page — workflow editor with real execution.
import 'dart:convert';
import 'package:flutter/material.dart';
import '../api/flowforge_api.dart';
import '../theme/flowforge_theme.dart';
import '../widgets/ff_widgets.dart';

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

  // Node list editing
  List<WorkflowNode> _nodes = [];
  List<WorkflowEdge> _edges = [];

  @override
  void initState() {
    super.initState();
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
          const SnackBar(content: Text('已保存')),
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
      // Save first
      await _save();
      // Then execute
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
    String selectedType = 'log';
    
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
              items: const [
                DropdownMenuItem(value: 'log', child: Text('日志输出')),
                DropdownMenuItem(value: 'delay', child: Text('延时等待')),
                DropdownMenuItem(value: 'http', child: Text('HTTP 请求')),
                DropdownMenuItem(value: 'script', child: Text('脚本')),
                DropdownMenuItem(value: 'shell', child: Text('Shell 命令')),
              ],
              onChanged: (v) => selectedType = v ?? 'log',
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
                    config: selectedType == 'log' ? {'level': 'info'} : {},
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

  void _removeNode(int index) {
    final nodeId = _nodes[index].id;
    setState(() {
      _nodes.removeAt(index);
      _edges.removeWhere((e) => e.from == nodeId || e.to == nodeId);
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
                items: _nodes.map((n) => DropdownMenuItem(value: n.id, child: Text(n.id))).toList(),
                onChanged: (v) => setDialogState(() => fromNode = v ?? fromNode),
              ),
              const SizedBox(height: 12),
              DropdownButtonFormField<String>(
                value: toNode,
                decoration: const InputDecoration(labelText: '到', border: OutlineInputBorder()),
                items: _nodes.map((n) => DropdownMenuItem(value: n.id, child: Text(n.id))).toList(),
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
                Expanded(flex: 3, child: _buildEditorPanel(theme, ext)),
                const SizedBox(width: FlowForgeSpacing.md),
                Expanded(flex: 2, child: _buildOutputPanel(theme, ext)),
              ],
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
          Icon(Icons.error_outline, size: 48, color: theme.colorScheme.error),
          const SizedBox(height: FlowForgeSpacing.md),
          FfText(_error!, fontSize: 14),
          const SizedBox(height: FlowForgeSpacing.md),
          FfButton(
            onTap: () => _loadWorkflow(widget.workflowId!),
            builder: (ctx, _) => const FfText('重试'),
          ),
        ],
      ),
    );
  }

  Widget _buildTopBar(ThemeData theme, FlowForgeThemeExtension ext) {
    return SizedBox(
      height: ext.topBarHeight,
      child: Row(
        children: [
          SizedBox(
            width: 200,
            child: TextField(
              controller: _nameController,
              style: const TextStyle(fontSize: 18, fontWeight: FontWeight.w600),
              decoration: const InputDecoration(border: InputBorder.none, isDense: true),
            ),
          ),
          const Spacer(),
          // Save button
          FfButton(
            onTap: _isSaving ? null : _save,
            builder: (ctx, hovering) => Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(_isSaving ? Icons.hourglass_empty : Icons.save, size: 16,
                    color: hovering ? ext.brandColor : null),
                  const SizedBox(width: 4),
                  FfText(_isSaving ? '保存中...' : '保存', fontSize: 13),
                ],
              ),
            ),
          ),
          const SizedBox(width: FlowForgeSpacing.sm),
          // Execute button
          FfButton(
            onTap: _isExecuting ? null : _execute,
            builder: (ctx, hovering) => Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
              decoration: BoxDecoration(
                color: _isExecuting ? Colors.grey : ext.brandColor,
                borderRadius: BorderRadius.circular(FlowForgeRadius.md),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(_isExecuting ? Icons.hourglass_empty : Icons.play_arrow, size: 16, color: Colors.white),
                  const SizedBox(width: 4),
                  FfText(_isExecuting ? '执行中...' : '执行', fontSize: 13, color: Colors.white),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildEditorPanel(ThemeData theme, FlowForgeThemeExtension ext) {
    return Container(
      decoration: BoxDecoration(
        border: Border.all(color: ext.borderColor),
        borderRadius: BorderRadius.circular(FlowForgeRadius.lg),
      ),
      child: Column(
        children: [
          // Header
          Container(
            padding: const EdgeInsets.all(FlowForgeSpacing.sm),
            decoration: BoxDecoration(
              color: ext.surfaceColor,
              borderRadius: const BorderRadius.only(
                topLeft: Radius.circular(FlowForgeRadius.lg),
                topRight: Radius.circular(FlowForgeRadius.lg),
              ),
            ),
            child: Row(
              children: [
                const Icon(Icons.account_tree, size: 16),
                const SizedBox(width: FlowForgeSpacing.sm),
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
                ? Center(
                    child: FfText('点击 + 添加节点', fontSize: 13,
                      color: theme.colorScheme.onSurface.withValues(alpha: 0.4)),
                  )
                : ListView.builder(
                    padding: const EdgeInsets.all(FlowForgeSpacing.sm),
                    itemCount: _nodes.length,
                    itemBuilder: (ctx, i) => _buildNodeCard(i, theme, ext),
                  ),
          ),
          // Edges summary
          if (_edges.isNotEmpty) ...[
            const FfDivider(),
            Padding(
              padding: const EdgeInsets.all(FlowForgeSpacing.sm),
              child: FfText(
                '连接: ${_edges.map((e) => '${e.from}→${e.to}').join(', ')}',
                fontSize: 11,
                color: theme.colorScheme.onSurface.withValues(alpha: 0.5),
              ),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildNodeCard(int index, ThemeData theme, FlowForgeThemeExtension ext) {
    final node = _nodes[index];
    return Card(
      margin: const EdgeInsets.only(bottom: 4),
      child: ListTile(
        dense: true,
        leading: Icon(_nodeIcon(node.type), size: 20, color: ext.brandColor),
        title: FfText(node.id, fontSize: 13, fontWeight: FontWeight.w600),
        subtitle: FfText('${node.type}  ${_configSummary(node.config)}', fontSize: 11,
          color: theme.colorScheme.onSurface.withValues(alpha: 0.5)),
        trailing: FfButton(
          onTap: () => _removeNode(index),
          builder: (ctx, hovering) => Icon(Icons.close, size: 16,
            color: hovering ? Colors.red : theme.colorScheme.onSurface.withValues(alpha: 0.3)),
        ),
      ),
    );
  }

  IconData _nodeIcon(String type) {
    switch (type) {
      case 'log': return Icons.text_snippet;
      case 'delay': return Icons.timer;
      case 'http': return Icons.http;
      case 'script': return Icons.code;
      case 'shell': return Icons.terminal;
      default: return Icons.circle;
    }
  }

  String _configSummary(Map<String, dynamic> config) {
    if (config.isEmpty) return '';
    final entries = config.entries.take(2).map((e) => '${e.key}=${e.value}').join(', ');
    return entries.length > 40 ? '${entries.substring(0, 37)}...' : entries;
  }

  Widget _buildOutputPanel(ThemeData theme, FlowForgeThemeExtension ext) {
    return Container(
      decoration: BoxDecoration(
        border: Border.all(color: ext.borderColor),
        borderRadius: BorderRadius.circular(FlowForgeRadius.lg),
      ),
      child: Column(
        children: [
          Container(
            padding: const EdgeInsets.all(FlowForgeSpacing.sm),
            decoration: BoxDecoration(
              color: ext.surfaceColor,
              borderRadius: const BorderRadius.only(
                topLeft: Radius.circular(FlowForgeRadius.lg),
                topRight: Radius.circular(FlowForgeRadius.lg),
              ),
            ),
            child: const Row(
              children: [
                Icon(Icons.output, size: 16),
                SizedBox(width: FlowForgeSpacing.sm),
                FfText('输出', fontSize: 12, fontWeight: FontWeight.w600),
              ],
            ),
          ),
          const FfDivider(),
          Expanded(
            child: Padding(
              padding: const EdgeInsets.all(FlowForgeSpacing.md),
              child: SingleChildScrollView(
                child: SelectableText(
                  _output.isEmpty ? '执行结果将显示在这里' : _output,
                  style: TextStyle(
                    fontFamily: 'monospace',
                    fontSize: 12,
                    color: _output.isEmpty
                        ? theme.colorScheme.onSurface.withValues(alpha: 0.4)
                        : _output.startsWith('✅')
                            ? Colors.green.shade700
                            : _output.startsWith('❌')
                                ? Colors.red.shade700
                                : theme.colorScheme.onSurface,
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  @override
  void dispose() {
    _nameController.dispose();
    super.dispose();
  }
}
