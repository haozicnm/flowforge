// Visual canvas editor — drag nodes, draw edges, zoom/pan, undo/redo, minimap.
import 'dart:convert';
import 'dart:math' as math;
import 'package:flutter/material.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import '../api/flowforge_api.dart';
import '../theme/flowforge_theme.dart';

/// Callback when canvas nodes/edges change.
typedef CanvasChanged = void Function(List<WorkflowNode> nodes, List<WorkflowEdge> edges);

/// A snapshot of canvas state for undo/redo.
class _CanvasSnapshot {
  final List<WorkflowNode> nodes;
  final List<WorkflowEdge> edges;
  _CanvasSnapshot(this.nodes, this.edges);

  _CanvasSnapshot copy() => _CanvasSnapshot(
    nodes.map((n) => WorkflowNode(
      id: n.id, type: n.type, label: n.label,
      config: n.config, position: {'x': n.positionX, 'y': n.positionY},
    )).toList(),
    edges.map((e) => WorkflowEdge(
      from: e.from, fromPort: e.fromPort, to: e.to,
    )).toList(),
  );
}

/// A group of nodes rendered as a colored box on the canvas.
class NodeGroup {
  final String id;
  String label;
  final Set<String> nodeIds;
  final Color color;
  bool collapsed;

  NodeGroup({
    required this.id,
    required this.label,
    required this.nodeIds,
    this.color = const Color(0xFF4FC3F7),
    this.collapsed = false,
  });
}

class CanvasEditor extends StatefulWidget {
  final List<WorkflowNode> nodes;
  final List<WorkflowEdge> edges;
  final List<NodeTypeDef> nodeTypes;
  final CanvasChanged? onChanged;
  final String? selectedNodeId;
  final ValueChanged<String?>? onNodeSelected;
  /// Set of node IDs that have breakpoints.
  final Set<String> breakpoints;
  /// Callback when a breakpoint is toggled.
  final ValueChanged<String>? onBreakpointToggle;
  /// Set of node IDs that failed during execution.
  final Set<String> failedNodes;
  /// Map of node ID → execution duration in ms.
  final Map<String, int> nodeDurations;
  /// Map of node ID → output data (for hover tooltip).
  final Map<String, Map<String, dynamic>> nodeOutputs;
  /// Node groups for visual grouping.
  final List<NodeGroup> groups;
  /// Callback when groups change.
  final ValueChanged<List<NodeGroup>>? onGroupsChanged;

  const CanvasEditor({
    super.key,
    required this.nodes,
    required this.edges,
    this.nodeTypes = const [],
    this.onChanged,
    this.selectedNodeId,
    this.onNodeSelected,
    this.breakpoints = const {},
    this.onBreakpointToggle,
    this.failedNodes = const {},
    this.nodeDurations = const {},
    this.nodeOutputs = const {},
    this.groups = const [],
    this.onGroupsChanged,
  });

  @override
  State<CanvasEditor> createState() => _CanvasEditorState();
}

class _CanvasEditorState extends State<CanvasEditor> {
  // Canvas transform
  Offset _panOffset = Offset.zero;
  double _scale = 1.0;

  // Dragging state
  String? _draggingNodeId;
  Offset _dragStartLocal = Offset.zero;
  Offset _dragStartNodePos = Offset.zero;

  // Edge drawing state
  String? _edgeFromNodeId;
  String? _edgeFromPort;
  Offset? _edgeDrawEnd;

  // Hover state
  String? _hoveredNodeId;
  Offset _hoverPosition = Offset.zero;

  // Multi-select state (box selection)
  final Set<String> _selectedNodeIds = {};

  // Node size
  static const double _nodeWidth = 180.0;
  static const double _nodeHeight = 64.0;
  static const double _portRadius = 7.0;

  // Undo/redo history
  final List<_CanvasSnapshot> _undoStack = [];
  final List<_CanvasSnapshot> _redoStack = [];
  static const int _maxHistory = 50;

  /// Save current state to undo stack before a mutation.
  void _saveSnapshot() {
    _undoStack.add(_CanvasSnapshot(widget.nodes, widget.edges).copy());
    if (_undoStack.length > _maxHistory) _undoStack.removeAt(0);
    _redoStack.clear(); // new action clears redo
  }

  void _undo() {
    if (_undoStack.isEmpty) return;
    final snapshot = _undoStack.removeLast();
    _redoStack.add(_CanvasSnapshot(widget.nodes, widget.edges).copy());
    _restoreSnapshot(snapshot);
  }

  void _redo() {
    if (_redoStack.isEmpty) return;
    final snapshot = _redoStack.removeLast();
    _undoStack.add(_CanvasSnapshot(widget.nodes, widget.edges).copy());
    _restoreSnapshot(snapshot);
  }

  void _restoreSnapshot(_CanvasSnapshot snapshot) {
    widget.nodes.clear();
    widget.nodes.addAll(snapshot.nodes);
    widget.edges.clear();
    widget.edges.addAll(snapshot.edges);
    widget.onChanged?.call(widget.nodes, widget.edges);
    setState(() {});
  }

  void _handleKey(LogicalKeyboardKey key, bool control, bool shift) {
    if (control && key == LogicalKeyboardKey.keyZ && !shift) _undo();
    if (control && key == LogicalKeyboardKey.keyY) _redo();
    if (control && key == LogicalKeyboardKey.keyZ && shift) _redo();
    if (key == LogicalKeyboardKey.delete || key == LogicalKeyboardKey.backspace) {
      _deleteSelected();
    }
    if (control && key == LogicalKeyboardKey.keyG) {
      _groupSelectedNodes();
    }
    if (control && key == LogicalKeyboardKey.keyA) {
      // Select all nodes
      setState(() {
        _selectedNodeIds.clear();
        _selectedNodeIds.addAll(widget.nodes.map((n) => n.id));
      });
    }
    if (key == LogicalKeyboardKey.escape) {
      setState(() {
        _selectedNodeIds.clear();
      });
    }
  }

  void _deleteSelected() {
    if (widget.selectedNodeId == null) return;
    _saveSnapshot();
    final id = widget.selectedNodeId!;
    widget.nodes.removeWhere((n) => n.id == id);
    widget.edges.removeWhere((e) => e.from == id || e.to == id);
    widget.onNodeSelected?.call(null);
    widget.onChanged?.call(widget.nodes, widget.edges);
    setState(() {});
  }

  /// Group selected nodes into a visual group (Ctrl+G).
  void _groupSelectedNodes() {
    final ids = _selectedNodeIds.isNotEmpty
        ? Set<String>.from(_selectedNodeIds)
        : widget.selectedNodeId != null
            ? {widget.selectedNodeId!}
            : <String>{};
    if (ids.length < 2) return; // need at least 2 nodes

    _saveSnapshot();
    final groupId = 'group_${DateTime.now().millisecondsSinceEpoch}';
    final colors = [
      const Color(0xFF4FC3F7), // light blue
      const Color(0xFF81C784), // green
      const Color(0xFFFFB74D), // orange
      const Color(0xFFCE93D8), // purple
      const Color(0xFFEF5350), // red
    ];
    final color = colors[widget.groups.length % colors.length];

    final group = NodeGroup(
      id: groupId,
      label: '分组 ${widget.groups.length + 1}',
      nodeIds: ids,
      color: color,
    );

    final newGroups = [...widget.groups, group];
    widget.onGroupsChanged?.call(newGroups);
    setState(() {
      _selectedNodeIds.clear();
    });
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final ext = theme.extension<FlowForgeThemeExtension>()!;

    return KeyboardListener(
      focusNode: FocusNode()..requestFocus(),
      onKeyEvent: (e) {
        if (e is KeyDownEvent) {
          _handleKey(e.logicalKey, HardwareKeyboard.instance.isLogicalKeyPressed(LogicalKeyboardKey.controlLeft) || HardwareKeyboard.instance.isLogicalKeyPressed(LogicalKeyboardKey.controlRight), HardwareKeyboard.instance.isLogicalKeyPressed(LogicalKeyboardKey.shiftLeft) || HardwareKeyboard.instance.isLogicalKeyPressed(LogicalKeyboardKey.shiftRight));
        }
      },
      child: ClipRect(
        child: Stack(
          children: [
            // Main canvas
            GestureDetector(
              onPanStart: _onCanvasPanStart,
              onPanUpdate: _onCanvasPanUpdate,
              child: MouseRegion(
                cursor: _draggingNodeId != null ? SystemMouseCursors.grabbing : SystemMouseCursors.basic,
                child: CustomPaint(
                  painter: _CanvasPainter(
                    nodes: widget.nodes,
                    edges: widget.edges,
                    groups: widget.groups,
                    panOffset: _panOffset,
                    scale: _scale,
                    selectedNodeId: widget.selectedNodeId,
                    edgeFromNodeId: _edgeFromNodeId,
                    edgeFromPort: _edgeFromPort,
                    edgeDrawEnd: _edgeDrawEnd,
                    brandColor: ext.brandColor,
                    surfaceColor: ext.surfaceColor,
                    borderColor: ext.borderColor,
                    textColor: ext.textColor,
                  ),
                  size: Size.infinite,
                  child: GestureDetector(
                    behavior: HitTestBehavior.translucent,
                    onDoubleTap: () => _zoomTo(1.0), // double-click resets zoom
                    child: _buildNodes(ext, theme),
                  ),
                ),
              ),
            ),

            // Minimap
            Positioned(
              right: 16,
              bottom: 16,
              child: _Minimap(
                nodes: widget.nodes,
                edges: widget.edges,
                panOffset: _panOffset,
                scale: _scale,
                canvasSize: MediaQuery.of(context).size,
                nodeWidth: _nodeWidth,
                nodeHeight: _nodeHeight,
                brandColor: ext.brandColor,
                borderColor: ext.borderColor,
                onTap: (worldPos) {
                  setState(() {
                    _panOffset = worldPos;
                  });
                },
              ),
            ),

            // Zoom controls
            Positioned(
              right: 16,
              top: 16,
              child: _ZoomControls(
                scale: _scale,
                onZoomIn: () => _zoomTo(_scale * 1.2),
                onZoomOut: () => _zoomTo(_scale / 1.2),
                onReset: () => _zoomTo(1.0),
              ),
            ),

            // Undo/redo indicator
            Positioned(
              left: 16,
              bottom: 16,
              child: _UndoRedoIndicator(
                canUndo: _undoStack.isNotEmpty,
                canRedo: _redoStack.isNotEmpty,
                onUndo: _undo,
                onRedo: _redo,
              ),
            ),

            // Hover output tooltip
            if (_hoveredNodeId != null && widget.nodeOutputs.containsKey(_hoveredNodeId))
              Positioned(
                left: _hoverPosition.dx + _nodeWidth * _scale + 8,
                top: _hoverPosition.dy,
                child: _OutputTooltip(
                  nodeId: _hoveredNodeId!,
                  outputs: widget.nodeOutputs[_hoveredNodeId]!,
                ),
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildNodes(FlowForgeThemeExtension ext, ThemeData theme) {
    return LayoutBuilder(
      builder: (context, constraints) {
        return Listener(
          onPointerSignal: (signal) {
            if (signal is PointerScrollEvent) {
              setState(() {
                final zoomDelta = signal.scrollDelta.dy > 0 ? 0.9 : 1.1;
                _scale = (_scale * zoomDelta).clamp(0.2, 3.0);
              });
            }
          },
          child: Stack(
            children: widget.nodes.map((node) {
              final screenPos = _worldToScreen(Offset(node.positionX, node.positionY));
              final isSelected = widget.selectedNodeId == node.id || _selectedNodeIds.contains(node.id);
              final isMultiSelected = _selectedNodeIds.contains(node.id) && widget.selectedNodeId != node.id;
              final isFailed = widget.failedNodes.contains(node.id);
              final duration = widget.nodeDurations[node.id];

              return Positioned(
                left: screenPos.dx,
                top: screenPos.dy,
                width: _nodeWidth * _scale,
                height: _nodeHeight * _scale,
                child: MouseRegion(
                  onEnter: (_) => setState(() {
                    _hoveredNodeId = node.id;
                    _hoverPosition = screenPos;
                  }),
                  onExit: (_) => setState(() => _hoveredNodeId = null),
                  child: GestureDetector(
                    onTap: () {
                      final shiftHeld = HardwareKeyboard.instance.isLogicalKeyPressed(LogicalKeyboardKey.shiftLeft) ||
                          HardwareKeyboard.instance.isLogicalKeyPressed(LogicalKeyboardKey.shiftRight);
                      if (shiftHeld) {
                        // Toggle multi-select
                        setState(() {
                          if (_selectedNodeIds.contains(node.id)) {
                            _selectedNodeIds.remove(node.id);
                          } else {
                            _selectedNodeIds.add(node.id);
                          }
                        });
                      } else {
                        _selectedNodeIds.clear();
                        widget.onNodeSelected?.call(node.id);
                      }
                    },
                    onDoubleTap: () => widget.onBreakpointToggle?.call(node.id),
                    onPanStart: (d) => _onNodeDragStart(node, d),
                    onPanUpdate: (d) => _onNodeDragUpdate(node, d),
                    onPanEnd: (_) => _onNodeDragEnd(node),
                    child: Container(
                    decoration: BoxDecoration(
                      color: isFailed
                          ? Colors.red.withValues(alpha: 0.1)
                          : isMultiSelected
                              ? Colors.purple.withValues(alpha: 0.1)
                              : isSelected
                                  ? ext.brandColor.withValues(alpha: 0.15)
                                  : ext.surfaceColor,
                      borderRadius: BorderRadius.circular(8 * _scale),
                      border: Border.all(
                        color: isFailed ? Colors.red : isMultiSelected ? Colors.purple : isSelected ? ext.brandColor : ext.borderColor,
                        width: isSelected || isFailed ? 2 : 1,
                      ),
                      boxShadow: [
                        BoxShadow(
                          color: Colors.black.withValues(alpha: 0.1),
                          blurRadius: 4,
                          offset: const Offset(0, 2),
                        ),
                      ],
                    ),
                    child: Stack(
                      children: [
                        // Node label
                        Center(
                          child: Padding(
                            padding: EdgeInsets.all(8 * _scale),
                            child: Text(
                              node.label.isNotEmpty ? node.label : node.type,
                              style: TextStyle(
                                fontSize: 12 * _scale,
                                fontWeight: FontWeight.w500,
                                color: ext.textColor,
                              ),
                              overflow: TextOverflow.ellipsis,
                              textAlign: TextAlign.center,
                            ),
                          ),
                        ),

                        // Breakpoint indicator (red dot)
                        if (widget.breakpoints.contains(node.id))
                          Positioned(
                            top: 4 * _scale,
                            left: 4 * _scale,
                            child: Container(
                              width: 10 * _scale,
                              height: 10 * _scale,
                              decoration: const BoxDecoration(
                                color: Colors.red,
                                shape: BoxShape.circle,
                              ),
                            ),
                          ),

                        // Execution duration badge
                        if (duration != null)
                          Positioned(
                            bottom: 2 * _scale,
                            right: 4 * _scale,
                            child: Container(
                              padding: EdgeInsets.symmetric(horizontal: 4 * _scale, vertical: 1 * _scale),
                              decoration: BoxDecoration(
                                color: isFailed ? Colors.red.withValues(alpha: 0.2) : ext.brandColor.withValues(alpha: 0.15),
                                borderRadius: BorderRadius.circular(4 * _scale),
                              ),
                              child: Text(
                                duration >= 1000 ? '${(duration / 1000).toStringAsFixed(1)}s' : '${duration}ms',
                                style: TextStyle(
                                  fontSize: 9 * _scale,
                                  color: isFailed ? Colors.red : ext.brandColor,
                                  fontWeight: FontWeight.w500,
                                ),
                              ),
                            ),
                          ),

                        // Input port
                        Positioned(
                          left: -_portRadius * _scale,
                          top: (_nodeHeight * _scale) / 2 - _portRadius * _scale,
                          child: GestureDetector(
                            onPanStart: (d) => _onPortTap(node, 'in', isOutput: false),
                            child: Container(
                              width: _portRadius * 2 * _scale,
                              height: _portRadius * 2 * _scale,
                              decoration: BoxDecoration(
                                color: ext.brandColor,
                                shape: BoxShape.circle,
                              ),
                            ),
                          ),
                        ),

                        // Output port
                        Positioned(
                          right: -_portRadius * _scale,
                          top: (_nodeHeight * _scale) / 2 - _portRadius * _scale,
                          child: GestureDetector(
                            onPanStart: (d) => _onPortTap(node, 'out', isOutput: true),
                            onPanUpdate: (d) => _onEdgeDragUpdate(d),
                            onPanEnd: (d) => _onEdgeDragEnd(d),
                            child: Container(
                              width: _portRadius * 2 * _scale,
                              height: _portRadius * 2 * _scale,
                              decoration: BoxDecoration(
                                color: ext.brandColor,
                                shape: BoxShape.circle,
                              ),
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
                ),
              );
            }).toList(),
          ),
        );
      },
    );
  }

  Offset _worldToScreen(Offset world) {
    return Offset(
      world.dx * _scale + _panOffset.dx,
      world.dy * _scale + _panOffset.dy,
    );
  }

  void _zoomTo(double newScale) {
    setState(() => _scale = newScale.clamp(0.2, 3.0));
  }

  // --- Canvas panning ---
  void _onCanvasPanStart(DragStartDetails d) {
    // Only pan if not on a node
    bool onNode = false;
    for (final node in widget.nodes) {
      final sp = _worldToScreen(Offset(node.positionX, node.positionY));
      final rect = Rect.fromLTWH(sp.dx, sp.dy, _nodeWidth * _scale, _nodeHeight * _scale);
      if (rect.contains(d.localPosition)) {
        onNode = true;
        break;
      }
    }
    if (!onNode) {
      _dragStartLocal = d.localPosition;
      _dragStartNodePos = _panOffset;
    }
  }

  void _onCanvasPanUpdate(DragUpdateDetails d) {
    if (_draggingNodeId == null && _edgeFromNodeId == null) {
      setState(() {
        _panOffset = _dragStartNodePos + (d.localPosition - _dragStartLocal);
      });
    }
  }

  // --- Node dragging ---
  void _onNodeDragStart(WorkflowNode node, DragStartDetails d) {
    _saveSnapshot(); // save before drag
    _draggingNodeId = node.id;
    _dragStartLocal = d.localPosition;
    _dragStartNodePos = Offset(node.positionX, node.positionY);
    widget.onNodeSelected?.call(node.id);
  }

  void _onNodeDragUpdate(WorkflowNode node, DragUpdateDetails d) {
    if (_draggingNodeId != node.id) return;
    final delta = (d.localPosition - _dragStartLocal) / _scale;
    setState(() {
      node.positionX = _dragStartNodePos.dx + delta.dx;
      node.positionY = _dragStartNodePos.dy + delta.dy;
    });
    widget.onChanged?.call(widget.nodes, widget.edges);
  }

  void _onNodeDragEnd(WorkflowNode node) {
    _draggingNodeId = null;
  }

  // --- Port/edge interaction ---
  void _onPortTap(WorkflowNode node, String port, {required bool isOutput}) {
    if (isOutput) {
      _edgeFromNodeId = node.id;
      _edgeFromPort = port;
    }
  }

  void _onEdgeDragUpdate(DragUpdateDetails d) {
    if (_edgeFromNodeId != null) {
      setState(() => _edgeDrawEnd = d.localPosition);
    }
  }

  void _onEdgeDragEnd(DragEndDetails d) {
    if (_edgeFromNodeId == null) return;

    // Find target node under cursor
    final endPos = d.localPosition;
    for (final node in widget.nodes) {
      if (node.id == _edgeFromNodeId) continue;
      final screenPos = _worldToScreen(Offset(node.positionX, node.positionY));
      final rect = Rect.fromLTWH(screenPos.dx, screenPos.dy, _nodeWidth * _scale, _nodeHeight * _scale);
      if (rect.contains(endPos)) {
        // Check for duplicate
        final exists = widget.edges.any((e) => e.from == _edgeFromNodeId && e.to == node.id);
        if (!exists) {
          _saveSnapshot(); // save before adding edge
          widget.edges.add(WorkflowEdge(
            from: _edgeFromNodeId!,
            fromPort: _edgeFromPort ?? 'out',
            to: node.id,
          ));
          widget.onChanged?.call(widget.nodes, widget.edges);
        }
        break;
      }
    }

    setState(() {
      _edgeFromNodeId = null;
      _edgeFromPort = null;
      _edgeDrawEnd = null;
    });
  }
}



/// Undo/redo indicator buttons.
class _UndoRedoIndicator extends StatelessWidget {
  final bool canUndo;
  final bool canRedo;
  final VoidCallback onUndo;
  final VoidCallback onRedo;

  const _UndoRedoIndicator({
    required this.canUndo,
    required this.canRedo,
    required this.onUndo,
    required this.onRedo,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface.withValues(alpha: 0.9),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Theme.of(context).dividerColor),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          _btn(Icons.undo, canUndo, onUndo, 'Undo (Ctrl+Z)'),
          Container(width: 1, height: 28, color: Theme.of(context).dividerColor),
          _btn(Icons.redo, canRedo, onRedo, 'Redo (Ctrl+Y)'),
        ],
      ),
    );
  }

  Widget _btn(IconData icon, bool enabled, VoidCallback onTap, String tooltip) {
    return Tooltip(
      message: tooltip,
      child: InkWell(
        onTap: enabled ? onTap : null,
        borderRadius: BorderRadius.circular(8),
        child: Padding(
          padding: const EdgeInsets.all(6),
          child: Icon(icon, size: 18, color: enabled ? null : Colors.grey),
        ),
      ),
    );
  }
}

/// Zoom controls overlay.
class _ZoomControls extends StatelessWidget {
  final double scale;
  final VoidCallback onZoomIn;
  final VoidCallback onZoomOut;
  final VoidCallback onReset;

  const _ZoomControls({
    required this.scale,
    required this.onZoomIn,
    required this.onZoomOut,
    required this.onReset,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface.withValues(alpha: 0.9),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Theme.of(context).dividerColor),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          _btn(Icons.zoom_in, onZoomIn, 'Zoom In'),
          Container(width: 36, height: 1, color: Theme.of(context).dividerColor),
          InkWell(
            onTap: onReset,
            child: Padding(
              padding: const EdgeInsets.symmetric(vertical: 4, horizontal: 8),
              child: Text(
                '${(scale * 100).toInt()}%',
                style: const TextStyle(fontSize: 10, fontWeight: FontWeight.w500),
              ),
            ),
          ),
          Container(width: 36, height: 1, color: Theme.of(context).dividerColor),
          _btn(Icons.zoom_out, onZoomOut, 'Zoom Out'),
        ],
      ),
    );
  }

  Widget _btn(IconData icon, VoidCallback onTap, String tooltip) {
    return Tooltip(
      message: tooltip,
      child: InkWell(
        onTap: onTap,
        child: Padding(
          padding: const EdgeInsets.all(6),
          child: Icon(icon, size: 18),
        ),
      ),
    );
  }
}

/// Minimap — overview of all nodes in the workflow.
class _Minimap extends StatelessWidget {
  final List<WorkflowNode> nodes;
  final List<WorkflowEdge> edges;
  final Offset panOffset;
  final double scale;
  final Size canvasSize;
  final double nodeWidth;
  final double nodeHeight;
  final Color brandColor;
  final Color borderColor;
  final ValueChanged<Offset>? onTap;

  static const double _minimapWidth = 160;
  static const double _minimapHeight = 100;

  const _Minimap({
    required this.nodes,
    required this.edges,
    required this.panOffset,
    required this.scale,
    required this.canvasSize,
    required this.nodeWidth,
    required this.nodeHeight,
    required this.brandColor,
    required this.borderColor,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTapDown: (d) {
        // Convert minimap tap to world position
        if (nodes.isEmpty) return;
        final bounds = _getWorldBounds();
        final relX = d.localPosition.dx / _minimapWidth;
        final relY = d.localPosition.dy / _minimapHeight;
        final worldX = bounds.left + relX * bounds.width;
        final worldY = bounds.top + relY * bounds.height;
        // Center the view on the tapped position
        onTap?.call(Offset(
          -worldX * scale + canvasSize.width / 2,
          -worldY * scale + canvasSize.height / 2,
        ));
      },
      child: Container(
        width: _minimapWidth,
        height: _minimapHeight,
        decoration: BoxDecoration(
          color: Theme.of(context).colorScheme.surface.withValues(alpha: 0.85),
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: borderColor),
        ),
        child: CustomPaint(
          painter: _MinimapPainter(
            nodes: nodes,
            edges: edges,
            panOffset: panOffset,
            scale: scale,
            canvasSize: canvasSize,
            nodeWidth: nodeWidth,
            nodeHeight: nodeHeight,
            brandColor: brandColor,
            borderColor: borderColor,
          ),
        ),
      ),
    );
  }

  Rect _getWorldBounds() {
    if (nodes.isEmpty) return Rect.zero;
    double minX = double.infinity, minY = double.infinity;
    double maxX = -double.infinity, maxY = -double.infinity;
    for (final n in nodes) {
      minX = math.min(minX, n.positionX);
      minY = math.min(minY, n.positionY);
      maxX = math.max(maxX, n.positionX + nodeWidth);
      maxY = math.max(maxY, n.positionY + nodeHeight);
    }
    final padding = 100.0;
    return Rect.fromLTRB(minX - padding, minY - padding, maxX + padding, maxY + padding);
  }
}

class _MinimapPainter extends CustomPainter {
  final List<WorkflowNode> nodes;
  final List<WorkflowEdge> edges;
  final Offset panOffset;
  final double scale;
  final Size canvasSize;
  final double nodeWidth;
  final double nodeHeight;
  final Color brandColor;
  final Color borderColor;

  _MinimapPainter({
    required this.nodes,
    required this.edges,
    required this.panOffset,
    required this.scale,
    required this.canvasSize,
    required this.nodeWidth,
    required this.nodeHeight,
    required this.brandColor,
    required this.borderColor,
  });

  @override
  void paint(Canvas canvas, Size size) {
    if (nodes.isEmpty) return;

    // Calculate world bounds
    double minX = double.infinity, minY = double.infinity;
    double maxX = -double.infinity, maxY = -double.infinity;
    for (final n in nodes) {
      minX = math.min(minX, n.positionX);
      minY = math.min(minY, n.positionY);
      maxX = math.max(maxX, n.positionX + nodeWidth);
      maxY = math.max(maxY, n.positionY + nodeHeight);
    }
    final padding = 100.0;
    minX -= padding; minY -= padding;
    maxX += padding; maxY += padding;
    final worldW = maxX - minX;
    final worldH = maxY - minY;

    final sx = size.width / worldW;
    final sy = size.height / worldH;
    final s = math.min(sx, sy);

    // Draw edges
    final edgePaint = Paint()
      ..color = brandColor.withValues(alpha: 0.4)
      ..strokeWidth = 1
      ..style = PaintingStyle.stroke;
    for (final edge in edges) {
      final from = nodes.where((n) => n.id == edge.from).firstOrNull;
      final to = nodes.where((n) => n.id == edge.to).firstOrNull;
      if (from == null || to == null) continue;
      canvas.drawLine(
        Offset((from.positionX + nodeWidth - minX) * s, (from.positionY + nodeHeight / 2 - minY) * s),
        Offset((to.positionX - minX) * s, (to.positionY + nodeHeight / 2 - minY) * s),
        edgePaint,
      );
    }

    // Draw nodes
    final nodePaint = Paint()..color = brandColor.withValues(alpha: 0.6);
    for (final n in nodes) {
      final rect = Rect.fromLTWH(
        (n.positionX - minX) * s,
        (n.positionY - minY) * s,
        nodeWidth * s,
        nodeHeight * s,
      );
      canvas.drawRRect(RRect.fromRectAndRadius(rect, const Radius.circular(2)), nodePaint);
    }

    // Draw viewport rectangle
    final vpLeft = (-panOffset.dx / scale - minX) * s;
    final vpTop = (-panOffset.dy / scale - minY) * s;
    final vpW = (canvasSize.width / scale) * s;
    final vpH = (canvasSize.height / scale) * s;
    final vpPaint = Paint()
      ..color = brandColor.withValues(alpha: 0.3)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5;
    canvas.drawRect(Rect.fromLTWH(vpLeft, vpTop, vpW, vpH), vpPaint);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => true;
}

/// Output tooltip — shows when hovering over a node with execution outputs.
class _OutputTooltip extends StatelessWidget {
  final String nodeId;
  final Map<String, dynamic> outputs;

  const _OutputTooltip({required this.nodeId, required this.outputs});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;

    return Material(
      elevation: 8,
      borderRadius: BorderRadius.circular(8),
      child: Container(
        width: 250,
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: isDark ? const Color(0xFF1E1E2E) : Colors.white,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: isDark ? Colors.white24 : Colors.grey.shade300),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            // Header
            Text(
              '📤 $nodeId 输出',
              style: TextStyle(
                fontSize: 12,
                fontWeight: FontWeight.w600,
                color: isDark ? Colors.white70 : Colors.black87,
              ),
            ),
            const SizedBox(height: 8),
            // Output entries
            ...outputs.entries.map((e) {
              final valueStr = e.value is String
                  ? e.value as String
                  : const JsonEncoder.withIndent('  ').convert(e.value);
              final displayValue = valueStr.length > 100
                  ? '${valueStr.substring(0, 100)}...'
                  : valueStr;
              return Padding(
                padding: const EdgeInsets.only(bottom: 4),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      '  ${e.key}:',
                      style: TextStyle(
                        fontSize: 11,
                        fontWeight: FontWeight.w500,
                        color: isDark ? Colors.white54 : Colors.black54,
                      ),
                    ),
                    Text(
                      '  $displayValue',
                      style: TextStyle(
                        fontSize: 10,
                        fontFamily: 'monospace',
                        color: isDark ? Colors.white70 : Colors.black87,
                      ),
                      maxLines: 5,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ],
                ),
              );
            }),
          ],
        ),
      ),
    );
  }
}

/// Paints edges and grid.
class _CanvasPainter extends CustomPainter {
  final List<WorkflowNode> nodes;
  final List<WorkflowEdge> edges;
  final List<NodeGroup> groups;
  final Offset panOffset;
  final double scale;
  final String? selectedNodeId;
  final String? edgeFromNodeId;
  final String? edgeFromPort;
  final Offset? edgeDrawEnd;
  final Color brandColor;
  final Color surfaceColor;
  final Color borderColor;
  final Color textColor;

  _CanvasPainter({
    required this.nodes,
    required this.edges,
    this.groups = const [],
    required this.panOffset,
    required this.scale,
    this.selectedNodeId,
    this.edgeFromNodeId,
    this.edgeFromPort,
    this.edgeDrawEnd,
    required this.brandColor,
    required this.surfaceColor,
    required this.borderColor,
    required this.textColor,
  });

  static const double _nodeWidth = 180.0;
  static const double _nodeHeight = 64.0;

  @override
  void paint(Canvas canvas, Size size) {
    _drawGrid(canvas, size);

    // Draw groups (background rectangles behind nodes)
    for (final group in groups) {
      _drawGroup(canvas, group);
    }

    // Draw edges
    for (final edge in edges) {
      _drawEdge(canvas, edge);
    }

    // Draw edge being created
    if (edgeFromNodeId != null && edgeDrawEnd != null) {
      _drawTempEdge(canvas);
    }
  }

  /// Draw a group as a colored rectangle around its member nodes.
  void _drawGroup(Canvas canvas, NodeGroup group) {
    final groupNodes = nodes.where((n) => group.nodeIds.contains(n.id)).toList();
    if (groupNodes.isEmpty) return;

    // Calculate bounding box of group nodes
    double minX = double.infinity, minY = double.infinity;
    double maxX = -double.infinity, maxY = -double.infinity;
    for (final n in groupNodes) {
      minX = math.min(minX, n.positionX);
      minY = math.min(minY, n.positionY);
      maxX = math.max(maxX, n.positionX + _nodeWidth);
      maxY = math.max(maxY, n.positionY + _nodeHeight);
    }

    const padding = 20.0;
    final topLeft = _worldToScreen(Offset(minX - padding, minY - padding - 24));
    final bottomRight = _worldToScreen(Offset(maxX + padding, maxY + padding));
    final rect = Rect.fromPoints(topLeft, bottomRight);

    // Group background
    final bgPaint = Paint()
      ..color = group.color.withValues(alpha: 0.08)
      ..style = PaintingStyle.fill;
    canvas.drawRRect(RRect.fromRectAndRadius(rect, const Radius.circular(8)), bgPaint);

    // Group border
    final borderPaint = Paint()
      ..color = group.color.withValues(alpha: 0.4)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5;
    canvas.drawRRect(RRect.fromRectAndRadius(rect, const Radius.circular(8)), borderPaint);

    // Group label
    final textPainter = TextPainter(
      text: TextSpan(
        text: group.label,
        style: TextStyle(
          color: group.color.withValues(alpha: 0.8),
          fontSize: 11 * scale,
          fontWeight: FontWeight.w600,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    textPainter.paint(canvas, Offset(topLeft.dx + 8, topLeft.dy + 4));
  }

  void _drawGrid(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = borderColor.withValues(alpha: 0.15)
      ..strokeWidth = 0.5;

    final gridSize = 30.0 * scale;
    final offsetX = panOffset.dx % gridSize;
    final offsetY = panOffset.dy % gridSize;

    for (double x = offsetX; x < size.width; x += gridSize) {
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), paint);
    }
    for (double y = offsetY; y < size.height; y += gridSize) {
      canvas.drawLine(Offset(0, y), Offset(size.width, y), paint);
    }
  }

  void _drawEdge(Canvas canvas, WorkflowEdge edge) {
    final fromNode = nodes.where((n) => n.id == edge.from).firstOrNull;
    final toNode = nodes.where((n) => n.id == edge.to).firstOrNull;
    if (fromNode == null || toNode == null) return;

    final fromScreen = _worldToScreen(Offset(fromNode.positionX, fromNode.positionY));
    final toScreen = _worldToScreen(Offset(toNode.positionX, toNode.positionY));

    final start = Offset(
      fromScreen.dx + _nodeWidth * scale,
      fromScreen.dy + (_nodeHeight * scale) / 2,
    );
    final end = Offset(
      toScreen.dx,
      toScreen.dy + (_nodeHeight * scale) / 2,
    );

    _drawBezier(canvas, start, end, brandColor);
  }

  void _drawTempEdge(Canvas canvas) {
    final fromNode = nodes.where((n) => n.id == edgeFromNodeId).firstOrNull;
    if (fromNode == null) return;

    final fromScreen = _worldToScreen(Offset(fromNode.positionX, fromNode.positionY));
    final start = Offset(
      fromScreen.dx + _nodeWidth * scale,
      fromScreen.dy + (_nodeHeight * scale) / 2,
    );

    _drawBezier(canvas, start, edgeDrawEnd!, brandColor.withValues(alpha: 0.5));
  }

  void _drawBezier(Canvas canvas, Offset start, Offset end, Color color) {
    final ctrl = math.min((end.dx - start.dx).abs() * 0.5, 150.0);
    final path = Path()
      ..moveTo(start.dx, start.dy)
      ..cubicTo(start.dx + ctrl, start.dy, end.dx - ctrl, end.dy, end.dx, end.dy);

    final paint = Paint()
      ..color = color
      ..strokeWidth = 2.0 * scale
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.round;

    canvas.drawPath(path, paint);

    // Arrow at end
    final arrowPaint = Paint()
      ..color = color
      ..style = PaintingStyle.fill;
    final arrowPath = Path()
      ..moveTo(end.dx, end.dy)
      ..lineTo(end.dx - 8 * scale, end.dy - 4 * scale)
      ..lineTo(end.dx - 8 * scale, end.dy + 4 * scale)
      ..close();
    canvas.drawPath(arrowPath, arrowPaint);
  }

  Offset _worldToScreen(Offset world) {
    return Offset(
      world.dx * scale + panOffset.dx,
      world.dy * scale + panOffset.dy,
    );
  }

  @override
  bool shouldRepaint(covariant _CanvasPainter old) => true;
}
