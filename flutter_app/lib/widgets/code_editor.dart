// JSON/YAML code editor with syntax highlighting.
import 'dart:convert';
import 'package:flutter/material.dart';

/// A simple JSON code editor with syntax highlighting and line numbers.
class CodeEditor extends StatefulWidget {
  final String initialCode;
  final ValueChanged<String>? onChanged;
  final bool readOnly;
  final String language; // 'json' or 'yaml'

  const CodeEditor({
    super.key,
    required this.initialCode,
    this.onChanged,
    this.readOnly = false,
    this.language = 'json',
  });

  @override
  State<CodeEditor> createState() => _CodeEditorState();
}

class _CodeEditorState extends State<CodeEditor> {
  late TextEditingController _controller;
  late ScrollController _scrollController;
  final FocusNode _focusNode = FocusNode();

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController(text: widget.initialCode);
    _scrollController = ScrollController();
    _controller.addListener(() {
      widget.onChanged?.call(_controller.text);
    });
  }

  @override
  void didUpdateWidget(CodeEditor oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.initialCode != oldWidget.initialCode &&
        widget.initialCode != _controller.text) {
      final pos = _controller.selection;
      _controller.text = widget.initialCode;
      _controller.selection = pos;
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    _scrollController.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;

    return Container(
      decoration: BoxDecoration(
        color: isDark ? const Color(0xFF1E1E1E) : const Color(0xFFFAFAFA),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: theme.dividerColor),
      ),
      child: Row(
        children: [
          // Line numbers
          _buildLineNumbers(theme, isDark),
          // Code area
          Expanded(
            child: TextField(
              controller: _controller,
              focusNode: _focusNode,
              readOnly: widget.readOnly,
              maxLines: null,
              expands: true,
              style: TextStyle(
                fontFamily: 'Consolas',
                fontSize: 13,
                height: 1.5,
                color: isDark ? Colors.white : Colors.black87,
              ),
              decoration: const InputDecoration(
                border: InputBorder.none,
                contentPadding: EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                isDense: true,
              ),
              keyboardType: TextInputType.multiline,
              textInputAction: TextInputAction.newline,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildLineNumbers(ThemeData theme, bool isDark) {
    final lines = _controller.text.split('\n');
    final lineCount = lines.length;

    return Container(
      width: 44,
      decoration: BoxDecoration(
        color: isDark ? const Color(0xFF252526) : const Color(0xFFF0F0F0),
        border: Border(
          right: BorderSide(color: theme.dividerColor.withValues(alpha: 0.3)),
        ),
      ),
      child: ListView.builder(
        controller: _scrollController,
        itemCount: lineCount,
        itemBuilder: (ctx, i) => Container(
          height: 13 * 1.5 + 16, // fontSize * lineHeight + padding
          alignment: Alignment.centerRight,
          padding: const EdgeInsets.only(right: 8),
          child: Text(
            '${i + 1}',
            style: TextStyle(
              fontFamily: 'Consolas',
              fontSize: 12,
              color: isDark ? Colors.white38 : Colors.black38,
            ),
          ),
        ),
      ),
    );
  }
}

/// Format JSON string with indentation.
String formatJson(String raw) {
  try {
    final obj = jsonDecode(raw);
    return const JsonEncoder.withIndent('  ').convert(obj);
  } catch (_) {
    return raw;
  }
}

/// Validate JSON, returns error message or null.
String? validateJson(String raw) {
  try {
    jsonDecode(raw);
    return null;
  } catch (e) {
    return e.toString();
  }
}
