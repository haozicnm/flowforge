/// FlowForge Reusable Widget Library (ff_widgets)
///
/// AppFlowy pattern: every visual component is a reusable widget.
library;

import 'package:flutter/material.dart';
import '../theme/flowforge_theme.dart';

/// FfButton — AppFlowy FlowyHover pattern.
class FfButton extends StatefulWidget {
  final Widget Function(BuildContext context, bool isHovering) builder;
  final VoidCallback? onTap;
  final bool isSelected;
  final BorderRadius? radius;
  final Color? hoverColor;

  const FfButton({
    super.key,
    required this.builder,
    this.onTap,
    this.isSelected = false,
    this.radius,
    this.hoverColor,
  });

  @override
  State<FfButton> createState() => _FfButtonState();
}

class _FfButtonState extends State<FfButton> {
  bool _isHovering = false;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final ext = theme.extension<FlowForgeThemeExtension>()!;
    final radius = widget.radius ?? BorderRadius.circular(FlowForgeRadius.md);
    final hoverColor =
        widget.hoverColor ?? ext.brandColor.withValues(alpha: 0.05);

    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _isHovering = true),
      onExit: (_) => setState(() => _isHovering = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 100),
          decoration: BoxDecoration(
            color: widget.isSelected
                ? ext.brandColor.withValues(alpha: 0.1)
                : _isHovering
                    ? hoverColor
                    : Colors.transparent,
            borderRadius: radius,
          ),
          child: widget.builder(context, _isHovering),
        ),
      ),
    );
  }
}

/// FfText — AppFlowy FlowyText pattern.
class FfText extends StatelessWidget {
  final String text;
  final TextStyle? style;
  final Color? color;
  final double? fontSize;
  final FontWeight? fontWeight;
  final int? maxLines;
  final TextOverflow? overflow;

  const FfText(
    this.text, {
    super.key,
    this.style,
    this.color,
    this.fontSize,
    this.fontWeight,
    this.maxLines,
    this.overflow,
  });

  @override
  Widget build(BuildContext context) {
    return Text(
      text,
      style: (style ?? const TextStyle()).copyWith(
        color: color,
        fontSize: fontSize,
        fontWeight: fontWeight,
      ),
      maxLines: maxLines,
      overflow: overflow ?? TextOverflow.ellipsis,
    );
  }
}

/// FfDivider — thin, consistent dividers.
class FfDivider extends StatelessWidget {
  final Axis direction;
  final double? length;
  final Color? color;

  const FfDivider({
    super.key,
    this.direction = Axis.horizontal,
    this.length,
    this.color,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final ext = theme.extension<FlowForgeThemeExtension>()!;
    final dividerColor = color ?? ext.borderColor;

    if (direction == Axis.horizontal) {
      return Container(
        height: 1,
        width: length ?? double.infinity,
        color: dividerColor,
      );
    }
    return Container(
      width: 1,
      height: length ?? double.infinity,
      color: dividerColor,
    );
  }
}

/// FfHover — FlowyHover for transparent hover effect.
class FfHover extends StatefulWidget {
  final Widget child;
  final VoidCallback? onTap;
  final Color? hoverColor;
  final BorderRadius? borderRadius;

  const FfHover({
    super.key,
    required this.child,
    this.onTap,
    this.hoverColor,
    this.borderRadius,
  });

  @override
  State<FfHover> createState() => _FfHoverState();
}

class _FfHoverState extends State<FfHover> {
  bool _isHovering = false;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final ext = theme.extension<FlowForgeThemeExtension>()!;
    final hoverColor =
        widget.hoverColor ?? ext.brandColor.withValues(alpha: 0.05);

    return MouseRegion(
      cursor:
          widget.onTap != null ? SystemMouseCursors.click : MouseCursor.defer,
      onEnter: (_) => setState(() => _isHovering = true),
      onExit: (_) => setState(() => _isHovering = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 100),
          decoration: BoxDecoration(
            color: _isHovering ? hoverColor : Colors.transparent,
            borderRadius:
                widget.borderRadius ?? BorderRadius.circular(FlowForgeRadius.md),
          ),
          child: widget.child,
        ),
      ),
    );
  }
}
