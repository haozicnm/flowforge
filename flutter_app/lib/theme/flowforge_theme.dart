/// FlowForge Design System — Figma Specs
///
/// Following AppFlowy pattern: static const colors → theme → extension.
library;

import 'package:flutter/material.dart';

/// Brand colors.
abstract class FlowForgeColors {
  static const brand = Color(0xFF00B4D8);
  static const brandLight = Color(0xFF90E0EF);
  static const brandDark = Color(0xFF0077B6);
  static const surface = Color(0xFFF8F9FA);
  static const surfaceDark = Color(0xFF1A1A2E);
  static const textPrimary = Color(0xFF212529);
  static const textSecondary = Color(0xFF6C757D);
  static const border = Color(0xFFDEE2E6);
  static const success = Color(0xFF28A745);
  static const warning = Color(0xFFFFC107);
  static const error = Color(0xFFDC3545);
}

/// Spacing constants.
abstract class FlowForgeSpacing {
  static const double xs = 4;
  static const double sm = 8;
  static const double md = 16;
  static const double lg = 24;
  static const double xl = 32;
  static const double xxl = 48;
}

/// Border radius constants.
abstract class FlowForgeRadius {
  static const double sm = 4;
  static const double md = 8;
  static const double lg = 12;
  static const double xl = 16;
}

/// AppFlowy-style theme extension.
class FlowForgeThemeExtension extends ThemeExtension<FlowForgeThemeExtension> {
  final Color brandColor;
  final Color surfaceColor;
  final Color borderColor;
  final double sidebarWidth;
  final double topBarHeight;

  const FlowForgeThemeExtension({
    required this.brandColor,
    required this.surfaceColor,
    required this.borderColor,
    this.sidebarWidth = 220,
    this.topBarHeight = 52,
  });

  @override
  FlowForgeThemeExtension copyWith({
    Color? brandColor,
    Color? surfaceColor,
    Color? borderColor,
    double? sidebarWidth,
    double? topBarHeight,
  }) {
    return FlowForgeThemeExtension(
      brandColor: brandColor ?? this.brandColor,
      surfaceColor: surfaceColor ?? this.surfaceColor,
      borderColor: borderColor ?? this.borderColor,
      sidebarWidth: sidebarWidth ?? this.sidebarWidth,
      topBarHeight: topBarHeight ?? this.topBarHeight,
    );
  }

  @override
  FlowForgeThemeExtension lerp(
    covariant ThemeExtension<FlowForgeThemeExtension>? other,
    double t,
  ) {
    if (other is! FlowForgeThemeExtension) return this;
    return FlowForgeThemeExtension(
      brandColor: Color.lerp(brandColor, other.brandColor, t)!,
      surfaceColor: Color.lerp(surfaceColor, other.surfaceColor, t)!,
      borderColor: Color.lerp(borderColor, other.borderColor, t)!,
      sidebarWidth: sidebarWidth,
      topBarHeight: topBarHeight,
    );
  }
}

/// Light theme.
ThemeData buildLightTheme() {
  return ThemeData(
    useMaterial3: true,
    brightness: Brightness.light,
    colorScheme: ColorScheme.fromSeed(
      seedColor: FlowForgeColors.brand,
      brightness: Brightness.light,
    ),
    scaffoldBackgroundColor: FlowForgeColors.surface,
    extensions: const [
      FlowForgeThemeExtension(
        brandColor: FlowForgeColors.brand,
        surfaceColor: FlowForgeColors.surface,
        borderColor: FlowForgeColors.border,
      ),
    ],
  );
}

/// Dark theme.
ThemeData buildDarkTheme() {
  return ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    colorScheme: ColorScheme.fromSeed(
      seedColor: FlowForgeColors.brand,
      brightness: Brightness.dark,
    ),
    scaffoldBackgroundColor: FlowForgeColors.surfaceDark,
    extensions: const [
      FlowForgeThemeExtension(
        brandColor: FlowForgeColors.brandLight,
        surfaceColor: FlowForgeColors.surfaceDark,
        borderColor: Color(0xFF333333),
      ),
    ],
  );
}
