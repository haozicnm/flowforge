/// Reusable FlowForge widgets — inspired by AppFlowy's flowy_infra_ui.
library;

import 'package:flutter/material.dart';

/// Standard button with icon + text + hover state.
class FfButton extends StatefulWidget {
  final String text;
  final IconData? icon;
  final VoidCallback? onTap;
  final bool selected;
  final bool compact;

  const FfButton({
    super.key,
    required this.text,
    this.icon,
    this.onTap,
    this.selected = false,
    this.compact = false,
  });

  @override
  State<FfButton> createState() => _FfButtonState();
}

class _FfButtonState extends State<FfButton> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final bgColor = widget.selected
        ? theme.colorScheme.primary.withOpacity(0.1)
        : _hovered
            ? theme.hoverBg
            : Colors.transparent;

    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 150),
          padding: EdgeInsets.symmetric(
            horizontal: widget.compact ? 8 : 12,
            vertical: widget.compact ? 4 : 8,
          ),
          decoration: BoxDecoration(
            color: bgColor,
            borderRadius: BorderRadius.circular(6),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (widget.icon != null) ...[
                Icon(widget.icon, size: 18, color: theme.colorScheme.onSurface),
                const SizedBox(width: 8),
              ],
              Text(
                widget.text,
                style: theme.textTheme.bodyMedium?.copyWith(
                  fontWeight: widget.selected ? FontWeight.w600 : FontWeight.w400,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// Standard text with theme styling.
class FfText extends StatelessWidget {
  final String text;
  final TextStyle? style;
  final int? maxLines;
  final TextOverflow? overflow;

  const FfText(
    this.text, {
    super.key,
    this.style,
    this.maxLines,
    this.overflow,
  });

  @override
  Widget build(BuildContext context) {
    return Text(
      text,
      style: style ?? Theme.of(context).textTheme.bodyMedium,
      maxLines: maxLines,
      overflow: overflow,
    );
  }
}

/// Themed divider.
class FfDivider extends StatelessWidget {
  const FfDivider({super.key});

  @override
  Widget build(BuildContext context) {
    return Divider(
      height: 1,
      thickness: 1,
      color: Theme.of(context).dividerColor,
    );
  }
}

/// Hover state wrapper.
class FfHover extends StatefulWidget {
  final Widget Function(BuildContext context, bool hovered) builder;

  const FfHover({super.key, required this.builder});

  @override
  State<FfHover> createState() => _FfHoverState();
}

class _FfHoverState extends State<FfHover> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: widget.builder(context, _hovered),
    );
  }
}
