/// FlowForge theme system — inspired by AppFlowy's layered approach.
///
/// AppTheme (color tokens) → ColorScheme → ThemeData + FlowForgeThemeExtension
library;

import 'package:flutter/material.dart';

/// Semantic color tokens (light/dark).
class FlowForgeColors {
  // Brand
  static const Color primary = Color(0xFF00B4D8);
  static const Color primaryDark = Color(0xFF0096B7);

  // Surface
  static const Color surfaceLight = Color(0xFFFFFFFF);
  static const Color surfaceDark = Color(0xFF1A1A2E);
  static const Color sidebarLight = Color(0xFFF7F8FA);
  static const Color sidebarDark = Color(0xFF16213E);

  // Text
  static const Color textPrimary = Color(0xFF1A1A1A);
  static const Color textSecondary = Color(0xFF6B7280);
  static const Color textHint = Color(0xFF9CA3AF);

  // Interactive
  static const Color hoverLight = Color(0xFFF3F4F6);
  static const Color hoverDark = Color(0xFF2D2D4A);

  // Status
  static const Color success = Color(0xFF10B981);
  static const Color warning = Color(0xFFF59E0B);
  static const Color error = Color(0xFFEF4444);

  // Node palette
  static const Color nodeNetwork = Color(0xFF3B82F6);
  static const Color nodeFile = Color(0xFF8B5CF6);
  static const Color nodeLogic = Color(0xFFF59E0B);
  static const Color nodeSystem = Color(0xFF6B7280);
}

/// Build light theme.
ThemeData buildLightTheme() {
  const colors = FlowForgeColors;
  final colorScheme = ColorScheme.light(
    primary: colors.primary,
    onPrimary: Colors.white,
    surface: colors.surfaceLight,
    onSurface: colors.textPrimary,
    error: colors.error,
    onError: Colors.white,
  );

  return ThemeData(
    useMaterial3: true,
    colorScheme: colorScheme,
    fontFamily: 'Inter',
    scaffoldBackgroundColor: colors.surfaceLight,
    dividerColor: Colors.grey.shade200,
    cardTheme: CardThemeData(
      elevation: 0,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8),
        side: BorderSide(color: Colors.grey.shade200),
      ),
    ),
    navigationRailTheme: NavigationRailThemeData(
      backgroundColor: colors.sidebarLight,
      selectedIconTheme: const IconThemeData(color: colors.primary),
      unselectedIconTheme: IconThemeData(color: Colors.grey.shade500),
      indicatorColor: colors.primary.withOpacity(0.1),
    ),
    extensions: const [
      FlowForgeThemeExtension(
        sidebarBg: colors.sidebarLight,
        hoverBg: colors.hoverLight,
        success: colors.success,
        warning: colors.warning,
        nodeNetwork: colors.nodeNetwork,
        nodeFile: colors.nodeFile,
        nodeLogic: colors.nodeLogic,
        nodeSystem: colors.nodeSystem,
      ),
    ],
  );
}

/// Build dark theme.
ThemeData buildDarkTheme() {
  const colors = FlowForgeColors;
  final colorScheme = ColorScheme.dark(
    primary: colors.primary,
    onPrimary: Colors.white,
    surface: colors.surfaceDark,
    onSurface: Colors.white,
    error: colors.error,
    onError: Colors.white,
  );

  return ThemeData(
    useMaterial3: true,
    colorScheme: colorScheme,
    fontFamily: 'Inter',
    scaffoldBackgroundColor: colors.surfaceDark,
    dividerColor: Colors.white.withOpacity(0.1),
    cardTheme: CardThemeData(
      elevation: 0,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8),
        side: BorderSide(color: Colors.white.withOpacity(0.1)),
      ),
    ),
    navigationRailTheme: NavigationRailThemeData(
      backgroundColor: colors.sidebarDark,
      selectedIconTheme: const IconThemeData(color: colors.primary),
      unselectedIconTheme: IconThemeData(color: Colors.grey.shade600),
      indicatorColor: colors.primary.withOpacity(0.15),
    ),
    extensions: const [
      FlowForgeThemeExtension(
        sidebarBg: colors.sidebarDark,
        hoverBg: colors.hoverDark,
        success: colors.success,
        warning: colors.warning,
        nodeNetwork: colors.nodeNetwork,
        nodeFile: colors.nodeFile,
        nodeLogic: colors.nodeLogic,
        nodeSystem: colors.nodeSystem,
      ),
    ],
  );
}

/// Extension theme for FlowForge-specific semantic colors.
class FlowForgeThemeExtension extends ThemeExtension<FlowForgeThemeExtension> {
  final Color sidebarBg;
  final Color hoverBg;
  final Color success;
  final Color warning;
  final Color nodeNetwork;
  final Color nodeFile;
  final Color nodeLogic;
  final Color nodeSystem;

  const FlowForgeThemeExtension({
    required this.sidebarBg,
    required this.hoverBg,
    required this.success,
    required this.warning,
    required this.nodeNetwork,
    required this.nodeFile,
    required this.nodeLogic,
    required this.nodeSystem,
  });

  @override
  FlowForgeThemeExtension copyWith({
    Color? sidebarBg,
    Color? hoverBg,
    Color? success,
    Color? warning,
    Color? nodeNetwork,
    Color? nodeFile,
    Color? nodeLogic,
    Color? nodeSystem,
  }) {
    return FlowForgeThemeExtension(
      sidebarBg: sidebarBg ?? this.sidebarBg,
      hoverBg: hoverBg ?? this.hoverBg,
      success: success ?? this.success,
      warning: warning ?? this.warning,
      nodeNetwork: nodeNetwork ?? this.nodeNetwork,
      nodeFile: nodeFile ?? this.nodeFile,
      nodeLogic: nodeLogic ?? this.nodeLogic,
      nodeSystem: nodeSystem ?? this.nodeSystem,
    );
  }

  @override
  FlowForgeThemeExtension lerp(
    covariant ThemeExtension<FlowForgeThemeExtension>? other,
    double t,
  ) {
    if (other is! FlowForgeThemeExtension) return this;
    return FlowForgeThemeExtension(
      sidebarBg: Color.lerp(sidebarBg, other.sidebarBg, t)!,
      hoverBg: Color.lerp(hoverBg, other.hoverBg, t)!,
      success: Color.lerp(success, other.success, t)!,
      warning: Color.lerp(warning, other.warning, t)!,
      nodeNetwork: Color.lerp(nodeNetwork, other.nodeNetwork, t)!,
      nodeFile: Color.lerp(nodeFile, other.nodeFile, t)!,
      nodeLogic: Color.lerp(nodeLogic, other.nodeLogic, t)!,
      nodeSystem: Color.lerp(nodeSystem, other.nodeSystem, t)!,
    );
  }
}
