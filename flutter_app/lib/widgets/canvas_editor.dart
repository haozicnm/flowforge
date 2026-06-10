// Visual canvas editor — drag nodes, draw edges, zoom/pan.
import 'dart:math' as math;
import 'package:flutter/material.dart';
import '../api/flowforge_api.dart';
import '../theme/flowforge_theme.dart';
import '../widgets/ff_widgets.dart';

/// Callback when canvas nodes/edges change.
typedef CanvasChanged = void Function(List<WorkflowNode> nodes, List<WorkflowEdge> edges);

class CanvasEditor extends StatefulWidget {
  final List<WorkflowNode> nodes;
  final List<WorkflowEdge> edges;
  final List<NodeTypeDef> nodeTypes;
  final CanvasChanged? onChanged;
  final String? selectedNodeId;
  final ValueChanged<String?>? onNodeSelected;

  const CanvasEditor({
    super.key,
    required this.nodes,
    required this.edges,
    this.nodeTypes = const [],
    this.onChanged,
    this.selectedNodeId,
    this.onNodeSelected,
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

  // Node size
  static const double _nodeWidth = 180.0;
  static const double _nodeHeight = 64.0;
  static const double _portRadius = 7.0;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final ext = theme.extension<FlowForgeThemeExtension>()!;

    return ClipRect(
      child: GestureDetector(
        onPanStart: _onCanvasPanStart,
        onPanUpdate: _onCanvasPanUpdate,
        onPanEnd: _onCanvasPanEnd,
        onTapUp: (details) => widget.onNodeSelected?.call(null),
        child: MouseRegion(
          cursor: _draggingNodeId != null
              ? SystemMouseCursors.grabbing
              : _edgeFromNodeId != null
                  ? SystemMouseCursors.precise
                  : SystemMouseCursors.basic,
          child: CustomPaint(
            painter: _CanvasPainter(
              nodes: widget.nodes,
              edges: widget.edges,
              panOffset: _panOffset,
              scale: _scale,
              selectedNodeId: widget.selectedNodeId,
              edgeFromNodeId: _edgeFromNodeId,
              edgeFromPort: _edgeFromPort,
              edgeDrawEnd: _edgeDrawEnd,
              brandColor: ext.brandColor,
              surfaceColor: ext.surfaceColor,
              borderColor: theme.dividerColor,
              textColor: theme.colorScheme.onSurface,
            ),
            child: Stack(
              children: [
                // Node cards
                ...widget.nodes.map((node) => _buildNodeCard(node, theme, ext)),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildNodeCard(WorkflowNode node, ThemeData theme, FlowForgeThemeExtension ext) {
    final screenPos = _worldToScreen(Offset(node.positionX, node.positionY));
    final isSelected = widget.selectedNodeId == node.id;
    final nodeColor = _nodeColor(node.type, ext);

    return Positioned(
      left: screenPos.dx,
      top: screenPos.dy,
      width: _nodeWidth * _scale,
      height: _nodeHeight * _scale,
      child: GestureDetector(
        onTap: () => widget.onNodeSelected?.call(node.id),
        onPanStart: (d) => _onNodeDragStart(node, d),
        onPanUpdate: (d) => _onNodeDragUpdate(node, d),
        onPanEnd: (_) => _onNodeDragEnd(),
        child: Container(
          decoration: BoxDecoration(
            color: isSelected
                ? nodeColor.withValues(alpha: 0.15)
                : ext.surfaceColor,
            borderRadius: BorderRadius.circular(8 * _scale),
            border: Border.all(
              color: isSelected ? nodeColor : ext.borderColor,
              width: isSelected ? 2 : 1,
            ),
            boxShadow: isSelected
                ? [BoxShadow(color: nodeColor.withValues(alpha: 0.2), blurRadius: 8)]
                : null,
          ),
          child: Stack(
            children: [
              // Type color bar
              Positioned(
                left: 0, top: 0, bottom: 0,
                width: 4 * _scale,
                child: Container(
                  decoration: BoxDecoration(
                    color: nodeColor,
                    borderRadius: BorderRadius.only(
                      topLeft: Radius.circular(8 * _scale),
                      bottomLeft: Radius.circular(8 * _scale),
                    ),
                  ),
                ),
              ),
              // Label + type
              Positioned(
                left: 12 * _scale,
                right: 8 * _scale,
                top: 8 * _scale,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      node.label.isNotEmpty ? node.label : node.id,
                      style: TextStyle(
                        fontSize: 13 * _scale,
                        fontWeight: FontWeight.w600,
                        color: theme.colorScheme.onSurface,
                      ),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                    SizedBox(height: 2 * _scale),
                    Text(
                      _nodeDisplayName(node.type),
                      style: TextStyle(
                        fontSize: 11 * _scale,
                        color: nodeColor,
                      ),
                    ),
                  ],
                ),
              ),
              // Input port (left center)
              Positioned(
                left: -_portRadius,
                top: (_nodeHeight * _scale) / 2 - _portRadius,
                child: GestureDetector(
                  onPanStart: (d) => _onPortTap(node, 'in', isOutput: false),
                  child: _buildPort(nodeColor, false),
                ),
              ),
              // Output port (right center)
              Positioned(
                right: -_portRadius,
                top: (_nodeHeight * _scale) / 2 - _portRadius,
                child: GestureDetector(
                  onPanStart: (d) => _onPortTap(node, 'out', isOutput: true),
                  onPanUpdate: (d) => _onEdgeDragUpdate(d),
                  onPanEnd: (d) => _onEdgeDragEnd(d),
                  child: _buildPort(nodeColor, true),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildPort(Color color, bool isOutput) {
    return Container(
      width: _portRadius * 2,
      height: _portRadius * 2,
      decoration: BoxDecoration(
        color: color,
        shape: BoxShape.circle,
        border: Border.all(color: Colors.white, width: 2),
      ),
    );
  }

  Color _nodeColor(String type, FlowForgeThemeExtension ext) {
    switch (type) {
      case 'log': return Colors.teal;
      case 'http': return Colors.blue;
      case 'delay': return Colors.orange;
      case 'shell': return Colors.red;
      case 'script': return Colors.purple;
      case 'webhook': return Colors.green;
      default: return ext.brandColor;
    }
  }

  String _nodeDisplayName(String type) {
    switch (type) {
      case 'log': return '日志输出';
      case 'http': return 'HTTP 请求';
      case 'delay': return '延时等待';
      case 'shell': return 'Shell 命令';
      case 'script': return '脚本';
      case 'webhook': return 'Webhook';
      default: return type;
    }
  }

  // --- Coordinate transforms ---

  Offset _worldToScreen(Offset world) {
    return Offset(
      world.dx * _scale + _panOffset.dx,
      world.dy * _scale + _panOffset.dy,
    );
  }

  Offset _screenToWorld(Offset screen) {
    return Offset(
      (screen.dx - _panOffset.dx) / _scale,
      (screen.dy - _panOffset.dy) / _scale,
    );
  }

  // --- Node dragging ---

  void _onNodeDragStart(WorkflowNode node, DragStartDetails d) {
    setState(() {
      _draggingNodeId = node.id;
      _dragStartLocal = d.localPosition;
      _dragStartNodePos = Offset(node.positionX, node.positionY);
    });
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

  void _onNodeDragEnd() {
    setState(() => _draggingNodeId = null);
  }

  // --- Canvas panning ---

  void _onCanvasPanStart(DragStartDetails d) {
    // Only pan if not clicking on a node
  }

  void _onCanvasPanUpdate(DragUpdateDetails d) {
    if (_draggingNodeId == null && _edgeFromNodeId == null) {
      setState(() => _panOffset += d.delta);
    }
  }

  void _onCanvasPanEnd(DragEndDetails d) {}

  // --- Edge drawing ---

  void _onPortTap(WorkflowNode node, String port, {required bool isOutput}) {
    if (isOutput) {
      setState(() {
        _edgeFromNodeId = node.id;
        _edgeFromPort = port;
      });
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

  // --- Zoom (mouse wheel) ---

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
  }
}

/// Paints edges and grid.
class _CanvasPainter extends CustomPainter {
  final List<WorkflowNode> nodes;
  final List<WorkflowEdge> edges;
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

    // Draw edges
    for (final edge in edges) {
      _drawEdge(canvas, edge);
    }

    // Draw edge being created
    if (edgeFromNodeId != null && edgeDrawEnd != null) {
      _drawTempEdge(canvas);
    }
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
