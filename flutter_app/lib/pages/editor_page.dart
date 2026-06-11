1|// Editor page — visual canvas + form editing + execution.
2|import 'dart:convert';
3|import 'package:easy_localization/easy_localization.dart';
4|import 'package:flutter/material.dart';
5|import '../api/flowforge_api.dart';
6|import '../theme/flowforge_theme.dart';
7|import '../widgets/ff_widgets.dart';
8|import '../widgets/canvas_editor.dart';
9|import '../widgets/code_editor.dart';
10|
11|class EditorPage extends StatefulWidget {
12|  final FlowForgeApi api;
13|  final String? workflowId;
14|
15|  const EditorPage({super.key, required this.api, this.workflowId});
16|
17|  @override
18|  State<EditorPage> createState() => _EditorPageState();
19|}
20|
21|class _EditorPageState extends State<EditorPage> {
22|  Workflow? _workflow;
23|  bool _loading = false;
24|  bool _isExecuting = false;
25|  bool _isSaving = false;
26|  String _output = '';
27|  String? _error;
28|  final _nameController = TextEditingController();
29|
30|  List<WorkflowNode> _nodes = [];
31|  List<WorkflowEdge> _edges = [];
32|  List<NodeTypeDef> _nodeTypes = [];
33|
34|  // View mode: canvas, form, or code
35|  int _viewMode = 0; // 0=canvas, 1=form, 2=code
36|  int _propsViewMode = 0; // 0=form, 1=json
37|  String? _selectedNodeId;
38|  String _codeJson = '';
39|  String _nodeSearchQuery = '';
40|  final Set<String> _breakpoints = {};
41|  final Set<String> _failedNodes = {};
42|  final Map<String, int> _nodeDurations = {};
43|  final Map<String, Map<String, dynamic>> _nodeOutputs = {};
44|  String? _hoveredNodeId;
45|  OverlayEntry? _searchOverlay;
46|
47|  @override
48|  void initState() {
49|    super.initState();
50|    _loadNodeTypes();
51|    if (widget.workflowId != null) {
52|      _loadWorkflow(widget.workflowId!);
53|    }
54|  }
55|
56|  @override
57|  void didUpdateWidget(EditorPage oldWidget) {
58|    super.didUpdateWidget(oldWidget);
59|    if (widget.workflowId != oldWidget.workflowId && widget.workflowId != null) {
60|      _loadWorkflow(widget.workflowId!);
61|    }
62|  }
63|
64|  Future<void> _loadNodeTypes() async {
65|    try {
66|      final types = await widget.api.nodeTypes();
67|      setState(() => _nodeTypes = types);
68|    } catch (_) {}
69|  }
70|
71|  Future<void> _loadWorkflow(String id) async {
72|    setState(() { _loading = true; _error = null; });
73|    try {
74|      final wf = await widget.api.getWorkflow(id);
75|      setState(() {
76|        _workflow = wf;
77|        _nodes = List.from(wf.nodes);
78|        _edges = List.from(wf.edges);
79|        _nameController.text = wf.name;
80|        _loading = false;
81|        _selectedNodeId = null;
82|      });
83|    } catch (e) {
84|      setState(() { _error = e.toString(); _loading = false; });
85|    }
86|  }
87|
88|  Future<void> _save() async {
89|    if (_workflow == null) return;
90|    setState(() => _isSaving = true);
91|    try {
92|      final updated = await widget.api.updateWorkflow(
93|        _workflow!.id,
94|        name: _nameController.text,
95|        nodes: _nodes,
96|        edges: _edges,
97|      );
98|      setState(() { _workflow = updated; _isSaving = false; });
99|      if (mounted) {
100|        ScaffoldMessenger.of(context).showSnackBar(
101|          const SnackBar(content: Text('已保存'), duration: Duration(seconds: 1)),
102|        );
103|      }
104|    } catch (e) {
105|      setState(() => _isSaving = false);
106|      if (mounted) {
107|        ScaffoldMessenger.of(context).showSnackBar(
108|          SnackBar(content: Text('保存失败: $e')),
109|        );
110|      }
111|    }
112|  }
113|
114|  Future<void> _execute() async {
115|    if (_workflow == null) return;
116|    setState(() { _isExecuting = true; _output = '执行中...'; _failedNodes.clear(); _nodeDurations.clear(); _nodeOutputs.clear(); });
117|    try {
118|      await _save();
119|      final result = await widget.api.executeWorkflow(_workflow!.id);
120|      final buf = StringBuffer();
121|      if (result.isSuccess) {
122|        buf.writeln('✅ 执行完成');
123|        buf.writeln();
124|        buf.writeln('节点执行顺序:');
125|        for (final nodeId in result.completed) {
126|          buf.writeln('  ✓ $nodeId');
127|        }
128|        if (result.failed.isNotEmpty) {
129|          buf.writeln();
130|          buf.writeln('失败节点:');
131|          for (final nodeId in result.failed) {
132|            buf.writeln('  ✗ $nodeId');
133|          }
134|        }
135|        if (result.nodeOutputs.isNotEmpty) {
136|          buf.writeln();
137|          buf.writeln('节点输出:');
138|          result.nodeOutputs.forEach((nodeId, outputs) {
139|            if (outputs is Map && outputs.isNotEmpty) {
140|              buf.writeln('  $nodeId:');
141|              outputs.forEach((port, value) {
142|                buf.writeln('    $port: ${const JsonEncoder.withIndent("  ").convert(value)}');
143|              });
144|            }
145|          });
146|        }
        setState(() {
          _failedNodes.addAll(result.failed);
          result.nodeOutputs.forEach((nodeId, outputs) {
            if (outputs is Map) {
              _nodeOutputs[nodeId] = Map<String, dynamic>.from(outputs);
            }
          });
        });
151|        buf.writeln('❌ 执行失败');
152|        buf.writeln(result.error ?? '未知错误');
        setState(() {
          _failedNodes.addAll(result.failed);
          result.nodeOutputs.forEach((nodeId, outputs) {
            if (outputs is Map) {
              _nodeOutputs[nodeId] = Map<String, dynamic>.from(outputs);
            }
          });
        });
157|      setState(() { _output = buf.toString(); _isExecuting = false; });
158|    } catch (e) {
159|      setState(() { _output = '❌ 执行出错: $e'; _isExecuting = false; });
160|    }
161|  }
162|

  /// Show Ctrl+F search overlay for finding nodes.
  void _showSearchOverlay() {
    _searchOverlay?.remove();
    _searchOverlay = OverlayEntry(
      builder: (ctx) {
        final theme = Theme.of(ctx);
        final ext = theme.extension<FlowForgeThemeExtension>()!;
        return Positioned(
          top: 80,
          left: MediaQuery.of(ctx).size.width / 2 - 200,
          child: Material(
            elevation: 12,
            borderRadius: BorderRadius.circular(8),
            child: Container(
              width: 400,
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: ext.surfaceColor,
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: ext.borderColor),
              ),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  TextField(
                    autofocus: true,
                    decoration: InputDecoration(
                      hintText: '搜索节点...',
                      prefixIcon: const Icon(Icons.search, size: 18),
                      isDense: true,
                      border: OutlineInputBorder(borderRadius: BorderRadius.circular(6)),
                    ),
                    style: const TextStyle(fontSize: 14),
                    onChanged: (q) {
                      // Filter nodes by label or type
                      final match = _nodes.where((n) =>
                        n.label.toLowerCase().contains(q.toLowerCase()) ||
                        n.type.toLowerCase().contains(q.toLowerCase())
                      ).firstOrNull;
                      if (match != null) {
                        setState(() => _selectedNodeId = match.id);
                      }
                    },
                    onSubmitted: (_) => _closeSearchOverlay(),
                  ),
                  const SizedBox(height: 8),
                  // Quick results
                  ..._nodes.take(5).map((n) => ListTile(
                    dense: true,
                    title: Text(n.label.isNotEmpty ? n.label : n.type, style: const TextStyle(fontSize: 13)),
                    subtitle: Text(n.type, style: const TextStyle(fontSize: 11)),
                    onTap: () {
                      setState(() => _selectedNodeId = n.id);
                      _closeSearchOverlay();
                    },
                  )),
                ],
              ),
            ),
          ),
        );
      },
    );
    Overlay.of(context).insert(_searchOverlay!);
  }

  void _closeSearchOverlay() {
    _searchOverlay?.remove();
    _searchOverlay = null;
  }

163|  void _addNode() {
164|    final idController = TextEditingController();
165|    String selectedType = _nodeTypes.isNotEmpty ? _nodeTypes.first.typeName : 'log';
166|
167|    showDialog(
168|      context: context,
169|      builder: (ctx) => AlertDialog(
170|        title: const Text('添加节点'),
171|        content: Column(
172|          mainAxisSize: MainAxisSize.min,
173|          children: [
174|            TextField(
175|              controller: idController,
176|              decoration: InputDecoration(labelText: '节点 ID', border: const OutlineInputBorder()),
177|            ),
178|            const SizedBox(height: 12),
179|            DropdownButtonFormField<String>(
180|              value: selectedType,
181|              decoration: InputDecoration(labelText: '类型', border: const OutlineInputBorder()),
182|              items: _nodeTypes.map((t) => DropdownMenuItem(
183|                value: t.typeName,
184|                child: Text(t.displayName),
185|              )).toList(),
186|              onChanged: (v) => selectedType = v ?? selectedType,
187|            ),
188|          ],
189|        ),
190|        actions: [
191|          TextButton(onPressed: () => Navigator.pop(ctx), child: const Text('取消')),
192|          ElevatedButton(
193|            onPressed: () {
194|              if (idController.text.isNotEmpty) {
195|                setState(() {
196|                  _nodes.add(WorkflowNode(
197|                    id: idController.text,
198|                    type: selectedType,
199|                    config: selectedType == 'log' ? {'level': 'info', 'message': ''} : {},
200|                    position: {'x': 100 + _nodes.length * 220.0, 'y': 100},
201|                  ));
202|                });
203|                Navigator.pop(ctx);
204|              }
205|            },
206|            child: Text('dashboard.create'.tr()),
207|          ),
208|        ],
209|      ),
210|    );
211|  }
212|
213|  void _removeNode(String nodeId) {
214|    setState(() {
215|      _nodes.removeWhere((n) => n.id == nodeId);
216|      _edges.removeWhere((e) => e.from == nodeId || e.to == nodeId);
217|      if (_selectedNodeId == nodeId) _selectedNodeId = null;
218|    });
219|  }
220|
221|  void _addEdge() {
222|    if (_nodes.length < 2) return;
223|    String fromNode = _nodes[0].id;
224|    String toNode = _nodes.length > 1 ? _nodes[1].id : _nodes[0].id;
225|
226|    showDialog(
227|      context: context,
228|      builder: (ctx) => StatefulBuilder(
229|        builder: (ctx, setDialogState) => AlertDialog(
230|          title: const Text('添加连接'),
231|          content: Column(
232|            mainAxisSize: MainAxisSize.min,
233|            children: [
234|              DropdownButtonFormField<String>(
235|                value: fromNode,
236|                decoration: const InputDecoration(labelText: '从', border: OutlineInputBorder()),
237|                items: _nodes.map((n) => DropdownMenuItem(value: n.id, child: Text(n.label.isNotEmpty ? n.label : n.id))).toList(),
238|                onChanged: (v) => setDialogState(() => fromNode = v ?? fromNode),
239|              ),
240|              const SizedBox(height: 12),
241|              DropdownButtonFormField<String>(
242|                value: toNode,
243|                decoration: const InputDecoration(labelText: '到', border: OutlineInputBorder()),
244|                items: _nodes.map((n) => DropdownMenuItem(value: n.id, child: Text(n.label.isNotEmpty ? n.label : n.id))).toList(),
245|                onChanged: (v) => setDialogState(() => toNode = v ?? toNode),
246|              ),
247|            ],
248|          ),
249|          actions: [
250|            TextButton(onPressed: () => Navigator.pop(ctx), child: const Text('取消')),
251|            ElevatedButton(
252|              onPressed: () {
253|                setState(() {
254|                  _edges.add(WorkflowEdge(from: fromNode, to: toNode));
255|                });
256|                Navigator.pop(ctx);
257|              },
258|              child: const Text('添加'),
259|            ),
260|          ],
261|        ),
262|      ),
263|    );
264|  }
265|
266|  @override
267|  Widget build(BuildContext context) {
268|    final theme = Theme.of(context);
269|    final ext = theme.extension<FlowForgeThemeExtension>()!;
270|
271|    if (widget.workflowId == null) {
272|      return _buildNoSelection(theme, ext);
273|    }
274|    if (_loading) return const Center(child: CircularProgressIndicator());
275|    if (_error != null) return _buildError(theme);
276|
277|    return Padding(
278|      padding: const EdgeInsets.all(FlowForgeSpacing.lg),
279|      child: Column(
280|        crossAxisAlignment: CrossAxisAlignment.start,
281|        children: [
282|          _buildTopBar(theme, ext),
283|          const SizedBox(height: FlowForgeSpacing.md),
284|          Expanded(
285|            child: Row(
286|              children: [
287|                // Left: node palette (only in canvas mode)
288|                if (_viewMode == 0) ...[
289|                  _buildNodePalette(theme, ext),
290|                  const SizedBox(width: FlowForgeSpacing.md),
291|                ],
292|                // Canvas / Form / Code editor
293|                Expanded(
294|                  flex: 3,
295|                  child: _viewMode == 0
296|                      ? _buildCanvas(ext)
297|                      : _viewMode == 1
298|                          ? _buildFormEditor(theme, ext)
299|                          : _buildCodeEditor(theme, ext),
300|                ),
301|                const SizedBox(width: FlowForgeSpacing.md),
302|                // Right panel: properties + output
303|                Expanded(
304|                  flex: 2,
305|                  child: Column(
306|                    children: [
307|                      if (_selectedNodeId != null) ...[
308|                        Expanded(child: _buildPropertiesPanel(theme, ext)),
309|                        const SizedBox(height: FlowForgeSpacing.md),
310|                      ],
311|                      Expanded(child: _buildOutputPanel(theme, ext)),
312|                    ],
313|                  ),
314|                ),
315|              ],
316|            ),
317|          ),
318|        ],
319|      ),
320|    );
321|  }
322|
323|  /// Left panel: categorized node palette with drag-to-add.
324|  Widget _buildNodePalette(ThemeData theme, FlowForgeThemeExtension ext) {
325|    // Group node types by category
326|    final categories = <String, List<NodeTypeDef>>{};
327|    for (final t in _nodeTypes) {
328|      categories.putIfAbsent(t.category, () => []).add(t);
329|    }
330|    // Sort categories
331|    final sortedCats = categories.keys.toList()..sort();
332|
333|    return Container(
334|      width: 200,
335|      decoration: BoxDecoration(
336|        color: ext.surfaceColor,
337|        borderRadius: BorderRadius.circular(8),
338|        border: Border.all(color: ext.borderColor),
339|      ),
340|      child: Column(
341|        crossAxisAlignment: CrossAxisAlignment.start,
342|        children: [
343|          // Header
344|          Padding(
345|            padding: const EdgeInsets.all(12),
346|            child: Row(
347|              children: [
348|                Icon(Icons.dashboard_customize_outlined, size: 16, color: ext.textColor),
349|                const SizedBox(width: 8),
350|                Text('节点', style: TextStyle(fontSize: 13, fontWeight: FontWeight.w600, color: ext.textColor)),
351|              ],
352|            ),
353|          ),
354|          Divider(height: 1, color: ext.borderColor),
355|          // Search field
356|          Padding(
357|            padding: const EdgeInsets.all(8),
358|            child: TextField(
359|              decoration: InputDecoration(
360|                hintText: '搜索节点...',
361|                hintStyle: TextStyle(fontSize: 12, color: ext.textColor.withValues(alpha: 0.4)),
362|                prefixIcon: Icon(Icons.search, size: 14, color: ext.textColor.withValues(alpha: 0.4)),
363|                isDense: true,
364|                contentPadding: const EdgeInsets.symmetric(vertical: 8, horizontal: 8),
365|                border: OutlineInputBorder(
366|                  borderRadius: BorderRadius.circular(6),
367|                  borderSide: BorderSide(color: ext.borderColor),
368|                ),
369|                filled: true,
370|                fillColor: ext.bgSecondary,
371|              ),
372|              style: TextStyle(fontSize: 12, color: ext.textColor),
373|              onChanged: (v) => setState(() => _nodeSearchQuery = v),
374|            ),
375|          ),
376|          // Node list grouped by category
377|          Expanded(
378|            child: ListView.builder(
379|              padding: const EdgeInsets.symmetric(horizontal: 4),
380|              itemCount: sortedCats.length,
381|              itemBuilder: (ctx, i) {
382|                final cat = sortedCats[i];
383|                final types = categories[cat]!;
384|                // Filter by search
385|                final filtered = _nodeSearchQuery.isEmpty
386|                    ? types
387|                    : types.where((t) =>
388|                        t.typeName.toLowerCase().contains(_nodeSearchQuery.toLowerCase()) ||
389|                        t.displayName.toLowerCase().contains(_nodeSearchQuery.toLowerCase())
390|                      ).toList();
391|                if (filtered.isEmpty) return const SizedBox.shrink();
392|
393|                return Theme(
394|                  data: theme.copyWith(dividerColor: Colors.transparent),
395|                  child: ExpansionTile(
396|                    initiallyExpanded: i < 3, // first 3 categories expanded
397|                    tilePadding: const EdgeInsets.symmetric(horizontal: 8),
398|                    childrenPadding: const EdgeInsets.only(left: 8, right: 4, bottom: 4),
399|                    title: Text(cat, style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600, color: ext.textColor)),
400|                    children: filtered.map((t) => _buildPaletteNode(t, ext)).toList(),
401|                  ),
402|                );
403|              },
404|            ),
405|          ),
406|        ],
407|      ),
408|    );
409|  }
410|
411|  Widget _buildPaletteNode(NodeTypeDef t, FlowForgeThemeExtension ext) {
412|    return DragTarget<Map<String, String>>(
413|      onAcceptWithDetails: (_) {},
414|      builder: (ctx, _, __) {
415|        return LongPressDraggable<Map<String, String>>(
416|          data: {'type': t.typeName, 'label': t.displayName},
417|          feedback: Material(
418|            elevation: 4,
419|            borderRadius: BorderRadius.circular(6),
420|            child: Container(
421|              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
422|              decoration: BoxDecoration(
423|                color: ext.brandColor.withValues(alpha: 0.2),
424|                borderRadius: BorderRadius.circular(6),
425|                border: Border.all(color: ext.brandColor),
426|              ),
427|              child: Text(t.displayName, style: TextStyle(fontSize: 12, fontWeight: FontWeight.w500, color: ext.textColor)),
428|            ),
429|          ),
430|          child: InkWell(
431|            onTap: () => _addNodeFromPalette(t),
432|            borderRadius: BorderRadius.circular(6),
433|            child: Padding(
434|              padding: const EdgeInsets.symmetric(vertical: 6, horizontal: 8),
435|              child: Row(
436|                children: [
437|                  Container(
438|                    width: 6, height: 6,
439|                    decoration: BoxDecoration(color: ext.brandColor, shape: BoxShape.circle),
440|                  ),
441|                  const SizedBox(width: 8),
442|                  Expanded(
443|                    child: Text(
444|                      t.displayName,
445|                      style: TextStyle(fontSize: 12, color: ext.textColor),
446|                      overflow: TextOverflow.ellipsis,
447|                    ),
448|                  ),
449|                ],
450|              ),
451|            ),
452|          ),
453|        );
454|      },
455|    );
456|  }
457|
458|  /// Add a node to the canvas from the palette (click or drop).
459|  void _addNodeFromPalette(NodeTypeDef t) {
460|    final id = '${t.typeName}_${DateTime.now().millisecondsSinceEpoch % 10000}';
461|    final node = WorkflowNode(
462|      id: id,
463|      type: t.typeName,
464|      label: t.displayName,
465|      config: {},
466|      positionX: 100 + _nodes.length * 30.0, // stagger
467|      positionY: 100 + _nodes.length * 20.0,
468|    );
469|    setState(() {
470|      _nodes.add(node);
471|      _selectedNodeId = id;
472|    });
473|  }
474|
475|  Widget _buildCanvas(FlowForgeThemeExtension ext) {
476|    return Container(
477|      decoration: BoxDecoration(
478|        color: ext.surfaceColor,
479|        borderRadius: BorderRadius.circular(8),
480|        border: Border.all(color: ext.borderColor),
481|      ),
482|      child: ClipRRect(
483|        borderRadius: BorderRadius.circular(8),
484|        child: CanvasEditor(
485|          nodes: _nodes,
486|          edges: _edges,
487|          nodeTypes: _nodeTypes,
488|          selectedNodeId: _selectedNodeId,
489|          onNodeSelected: (id) => setState(() => _selectedNodeId = id),
490|          onChanged: (nodes, edges) => setState(() {}),
491|          breakpoints: _breakpoints,
492|          onBreakpointToggle: (id) => setState(() {
493|            if (_breakpoints.contains(id)) {
494|              _breakpoints.remove(id);
495|            } else {
496|              _breakpoints.add(id);
497|            }
498|          }),
499|          failedNodes: _failedNodes,
500|          nodeDurations: _nodeDurations,
          nodeOutputs: _nodeOutputs,
501|