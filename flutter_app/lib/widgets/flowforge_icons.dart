/// FlowForge SVG Icon System
///
/// AppFlowy FlowySvg pattern: every icon is a named constant, theme-aware coloring.
/// Icons are hand-crafted SVG path data (derived from Lucide icons), rendered via CustomPaint.
/// Zero external dependencies.
library;

import 'package:flutter/material.dart';
import '../theme/flowforge_theme.dart';

// ─── Icon palette ────────────────────────────────────────────────

/// All available FlowForge icons.
enum FfIconName {
  // Nav
  bolt,
  workspaces,
  edit,
  settings,
  add,
  play,
  save,
  delete,
  close,
  chevronRight,
  check,
  info,
  error,
  link,
  formatAlignLeft,
  dashboard,
  list,
  code,
  // Node types
  textSnippet,
  http,
  timer,
  terminal,
  scriptCode,
  webhook,
  condition,
  loop,
  tryCatch,
  variable,
  jsonIcon,
  regexIcon,
  template,
  webNavigate,
  webClick,
  webInput,
  webExtract,
  webScreenshot,
  webWait,
  excel,
  docx,
  circle,
}

/// Map icon name → SVG path data string.
const Map<FfIconName, String> _pathData = {
  // ── Nav ──
  FfIconName.bolt: 'M13 2L3 14h9l-1 8 10-12h-9l1-8z',
  FfIconName.workspaces: 'M3 3h7v7H3V3zm11 0h7v7h-7V3zm0 11h7v7h-7v-7zM3 14h7v7H3v-7z',
  FfIconName.edit: 'M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7 M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z',
  FfIconName.settings: 'M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z',
  FfIconName.add: 'M12 5v14m-7-7h14',
  FfIconName.play: 'M5 3l14 9-14 9V3z',
  FfIconName.save: 'M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z M17 21v-8H7v8 M7 3v5h8',
  FfIconName.delete: 'M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2m3 0v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6h14',
  FfIconName.close: 'M18 6L6 18M6 6l12 12',
  FfIconName.chevronRight: 'M9 18l6-6-6-6',
  FfIconName.check: 'M20 6L9 17l-5-5',
  FfIconName.info: 'M12 16v-4m0-4h.01M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20z',
  FfIconName.error: 'M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z M12 9v4 M12 17h.01',
  FfIconName.link: 'M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71 M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71',
  FfIconName.formatAlignLeft: 'M21 15l-3-3 3-3 M12 9H3 M12 15H3 M20 3H3 M20 21H3',
  FfIconName.dashboard: 'M3 3h7v9H3V3zm14 4h4v4h-4V7zm0 6h4v8h-4v-8zm-14 5h7v4H3v-4z',
  FfIconName.list: 'M3 4h5v2H3V4zm0 7h5v2H3v-2zm0 7h5v2H3v-2z M11 5h10M11 12h10M11 19h10',
  FfIconName.code: 'M16 18l6-6-6-6 M8 6l-6 6 6 6',

  // ── Node types ──
  FfIconName.textSnippet: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6 M10 9H8m8 4H8m8 4H8',
  FfIconName.http: 'M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0z M3.6 9h16.8M3.6 15h9 M11.5 3a17 17 0 0 0 0 18 M12.5 3a16.98 16.98 0 0 1 2.53 9 16.98 16.98 0 0 1-2.53 9',
  FfIconName.timer: 'M12 22a7 7 0 1 0 0-14 7 7 0 0 0 0 14z M12 8v4l2 2 M12 2v2 M18.6 5.4l-1.4 1.4 M5.4 5.4L6.8 6.8',
  FfIconName.terminal: 'M4 17l6-6-6-6 M12 19h8',
  FfIconName.scriptCode: 'M16 2v4h4 M11 2H5a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h5 M15 22H5 M9 9l-2 2 2 2 M13 13h4',
  FfIconName.webhook: 'M18 16.98h-5.99c-1.1 0-1.95.94-2.48 1.9A4 4 0 0 1 2 17c.01-.7.2-1.4.57-2 M6 17l3.13-5.78c.53-.97.1-2.18-.5-3.19a4 4 0 1 1 6.89-4.06 M12 7l2 3.72c.52.97 1.42 1.63 2.49 1.63h.07 M16 16.5a3.5 3.5 0 1 0 0 7',
  FfIconName.condition: 'M4 4v16 M9 4v16 M14 12v8 M19 4v16',
  FfIconName.loop: 'M17 2l4 4-4 4 M3 11v-1a4 4 0 0 1 4-4h14 M7 22l-4-4 4-4 M21 13v1a4 4 0 0 1-4 4H3',
  FfIconName.tryCatch: 'M12 2v4 M12 18v4 M4.93 4.93l2.83 2.83 M16.24 16.24l2.83 2.83 M2 12h4 M18 12h4 M4.93 19.07l2.83-2.83 M16.24 7.76l2.83-2.83',
  FfIconName.variable: 'M8 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h3 M16 3h3a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-3 M7 8h2 M7 12h2 M7 16h3',
  FfIconName.jsonIcon: 'M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5z M14 2v6h6 M10 12a1 1 0 0 0-2 0v1a1 1 0 0 0 1 1 1 1 0 0 1 1 1v1a1 1 0 0 0 2 0 M16 12a1 1 0 0 1 2 0v1a1 1 0 0 1-1 1 1 1 0 0 0-1 1v1a1 1 0 0 1-2 0',
  FfIconName.regexIcon: 'M17 3v10 M12.67 5.5l8.66 5 M12.67 12.5l8.66-5 M9 17a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-1a2 2 0 0 1 2-2h.5A1.5 1.5 0 0 0 7 12.5V12a2 2 0 0 0-2-2H4',
  FfIconName.template: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6 M8 13h2 M8 17h5 M8 9h8',
  FfIconName.webNavigate: 'M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z',
  FfIconName.webClick: 'M3 3l7.07 16.97 2.51-7.39 7.39-2.51L3 3z M13 13l6 6',
  FfIconName.webInput: 'M4 7h16 M4 12h8 M4 17h16',
  FfIconName.webExtract: 'M17 9V7a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2h2 M9 5h10a2 2 0 0 1 2 2v6a2 2 0 0 1-2 2H9a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2z M17 9v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-3',
  FfIconName.webScreenshot: 'M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z M12 8v3m0 5h.01 M8.5 3l-3 3 M15.5 3l3 3 M3 8.5l3-3 M3 15.5l3 3 M21 8.5l-3-3 M21 15.5l-3 3',
  FfIconName.webWait: 'M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z M12 6v6l4 2',
  FfIconName.excel: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6 M8 13h2 M8 17h5 M8 9h8',
  FfIconName.docx: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6 M16 13H8 M16 17H8 M10 9H8',
  FfIconName.circle: 'M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z',
};

// ─── Color mapping per node category ─────────────────────────────

Color ffNodeColor(String type) {
  switch (type) {
    // 触发器
    case 'webhook': return const Color(0xFF10B981); // green
    // 流程控制
    case 'loop': return const Color(0xFFF59E0B);  // amber
    case 'condition': return const Color(0xFFF59E0B);
    case 'try_catch': return const Color(0xFFF59E0B);
    case 'delay': return const Color(0xFFF59E0B);
    // 数据处理
    case 'script': return const Color(0xFF8B5CF6); // purple
    case 'template': return const Color(0xFF8B5CF6);
    case 'json': return const Color(0xFF8B5CF6);
    case 'regex': return const Color(0xFF8B5CF6);
    case 'variable': return const Color(0xFF8B5CF6);
    // 网络
    case 'http': return const Color(0xFF3B82F6);  // blue
    // Web 自动化
    case 'web_navigate': return const Color(0xFFEC4899); // pink
    case 'web_click': return const Color(0xFFEC4899);
    case 'web_input': return const Color(0xFFEC4899);
    case 'web_extract': return const Color(0xFFEC4899);
    case 'web_screenshot': return const Color(0xFFEC4899);
    case 'web_wait': return const Color(0xFFEC4899);
    // 文件
    case 'excel_read': return const Color(0xFF14B8A6);  // teal
    case 'excel_write': return const Color(0xFF14B8A6);
    case 'docx_read': return const Color(0xFF6B7280);   // gray
    case 'docx_create': return const Color(0xFF6B7280);
    // 数据库
    case 'database': return const Color(0xFF0EA5E9);   // sky blue
    // 通知
    case 'notification': return const Color(0xFFF97316); // orange
    // 文件
    case 'file': return const Color(0xFF14B8A6);       // teal
    // 定时
    case 'cron': return const Color(0xFF84CC16);       // lime
    // 系统
    case 'shell': return const Color(0xFFEF4444); // red
    // 调试
    case 'log': return const Color(0xFF6B7280);
    default: return const Color(0xFF94A3B8); // slate
  }
}

FfIconName ffNodeIcon(String type) {
  switch (type) {
    case 'log': return FfIconName.textSnippet;
    case 'http': return FfIconName.http;
    case 'delay': return FfIconName.timer;
    case 'shell': return FfIconName.terminal;
    case 'script': return FfIconName.scriptCode;
    case 'webhook': return FfIconName.webhook;
    case 'condition': return FfIconName.condition;
    case 'loop': return FfIconName.loop;
    case 'try_catch': return FfIconName.tryCatch;
    case 'variable': return FfIconName.variable;
    case 'json': return FfIconName.jsonIcon;
    case 'regex': return FfIconName.regexIcon;
    case 'template': return FfIconName.template;
    case 'web_navigate': return FfIconName.webNavigate;
    case 'web_click': return FfIconName.webClick;
    case 'web_input': return FfIconName.webInput;
    case 'web_extract': return FfIconName.webExtract;
    case 'web_screenshot': return FfIconName.webScreenshot;
    case 'web_wait': return FfIconName.webWait;
    case 'excel_read': return FfIconName.excel;
    case 'excel_write': return FfIconName.excel;
    case 'docx_read': return FfIconName.docx;
    case 'docx_create': return FfIconName.docx;
    case 'database': return FfIconName.jsonIcon;
    case 'notification': return FfIconName.webhook;
    case 'file': return FfIconName.textSnippet;
    case 'cron': return FfIconName.timer;
    default: return FfIconName.circle;
  }
}

// ─── Path parser ──────────────────────────────────────────────────

class _SvgPathPainter extends CustomPainter {
  final String pathData;
  final Color color;

  _SvgPathPainter(this.pathData, this.color);

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = color
      ..style = PaintingStyle.fill
      ..strokeWidth = 1.5;

    final paths = _parsePaths(pathData, size);
    for (final path in paths) {
      canvas.drawPath(path, paint);
    }
  }

  @override
  bool shouldRepaint(_SvgPathPainter old) =>
      pathData != old.pathData || color != old.color;

  static List<Path> _parsePaths(String data, Size size) {
    final paths = <Path>[];
    final viewSize = 24.0;
    final sx = size.width / viewSize;
    final sy = size.height / viewSize;

    // Split on 'M' to find sub-paths (each starts with M)
    // But also handle multiple M commands within one string
    final parts = data.split(RegExp(r'(?=M\s)'));
    for (final part in parts) {
      if (part.isEmpty) continue;
      final path = Path();
      final tokens = part
          .trim()
          .replaceAll(',', ' ')
          .split(RegExp(r'\s+'))
          .where((t) => t.isNotEmpty)
          .toList();

      double? lastX, lastY;
      int i = 0;
      while (i < tokens.length) {
        final cmd = tokens[i];
        if (cmd == 'M') {
          final x = double.parse(tokens[i + 1]) * sx;
          final y = double.parse(tokens[i + 2]) * sy;
          path.moveTo(x, y);
          lastX = x; lastY = y;
          i += 3;
        } else if (cmd == 'm') {
          final x = (lastX ?? 0) + double.parse(tokens[i + 1]) * sx;
          final y = (lastY ?? 0) + double.parse(tokens[i + 2]) * sy;
          path.moveTo(x, y);
          lastX = x; lastY = y;
          i += 3;
        } else if (cmd == 'L') {
          final x = double.parse(tokens[i + 1]) * sx;
          final y = double.parse(tokens[i + 2]) * sy;
          path.lineTo(x, y);
          lastX = x; lastY = y;
          i += 3;
        } else if (cmd == 'l') {
          final x = (lastX ?? 0) + double.parse(tokens[i + 1]) * sx;
          final y = (lastY ?? 0) + double.parse(tokens[i + 2]) * sy;
          path.lineTo(x, y);
          lastX = x; lastY = y;
          i += 3;
        } else if (cmd == 'H') {
          final x = double.parse(tokens[i + 1]) * sx;
          path.lineTo(x, lastY ?? 0);
          lastX = x;
          i += 2;
        } else if (cmd == 'h') {
          final x = (lastX ?? 0) + double.parse(tokens[i + 1]) * sx;
          path.lineTo(x, lastY ?? 0);
          lastX = x;
          i += 2;
        } else if (cmd == 'V') {
          final y = double.parse(tokens[i + 1]) * sy;
          path.lineTo(lastX ?? 0, y);
          lastY = y;
          i += 2;
        } else if (cmd == 'v') {
          final y = (lastY ?? 0) + double.parse(tokens[i + 1]) * sy;
          path.lineTo(lastX ?? 0, y);
          lastY = y;
          i += 2;
        } else if (cmd == 'C') {
          final x1 = double.parse(tokens[i + 1]) * sx;
          final y1 = double.parse(tokens[i + 2]) * sy;
          final x2 = double.parse(tokens[i + 3]) * sx;
          final y2 = double.parse(tokens[i + 4]) * sy;
          final x3 = double.parse(tokens[i + 5]) * sx;
          final y3 = double.parse(tokens[i + 6]) * sy;
          path.cubicTo(x1, y1, x2, y2, x3, y3);
          lastX = x3; lastY = y3;
          i += 7;
        } else if (cmd == 'Q') {
          final x1 = double.parse(tokens[i + 1]) * sx;
          final y1 = double.parse(tokens[i + 2]) * sy;
          final x2 = double.parse(tokens[i + 3]) * sx;
          final y2 = double.parse(tokens[i + 4]) * sy;
          path.quadraticBezierTo(x1, y1, x2, y2);
          lastX = x2; lastY = y2;
          i += 5;
        } else if (cmd == 'A') {
          // rx ry x-rotation large-arc sweep x y
          final rx = double.parse(tokens[i + 1]) * sx;
          final ry = double.parse(tokens[i + 2]) * sy;
          // ignore rotation
          final sweep = double.parse(tokens[i + 5]) != 0;
          final x = double.parse(tokens[i + 6]) * sx;
          final y = double.parse(tokens[i + 7]) * sy;
          path.arcTo(
            Rect.fromCenter(center: Offset(lastX ?? 0, lastY ?? 0), width: rx * 2, height: ry * 2),
            0, sweep ? 180 : -180, false,
          );
          lastX = x; lastY = y;
          i += 8;
        } else if (cmd == 'Z' || cmd == 'z') {
          path.close();
          i += 1;
        } else {
          i += 1; // skip unknown
        }
      }
      paths.add(path);
    }
    return paths;
  }
}

// ─── FfSvg widget ─────────────────────────────────────────────────

class FfSvg extends StatelessWidget {
  final FfIconName icon;
  final double? size;
  final Color? color;
  final double? opacity;

  const FfSvg(
    this.icon, {
    super.key,
    this.size,
    this.color,
    this.opacity,
  });

  @override
  Widget build(BuildContext context) {
    final theme = FlowForgeThemeExtension.of(context);
    final pathData = _pathData[icon] ?? _pathData[FfIconName.circle]!;
    final sz = size ?? 20.0;
    final c = color ?? theme.icon.primary;

    return Opacity(
      opacity: opacity ?? 1.0,
      child: SizedBox(
        width: sz,
        height: sz,
        child: CustomPaint(
          painter: _SvgPathPainter(pathData, c),
          size: Size(sz, sz),
        ),
      ),
    );
  }
}
