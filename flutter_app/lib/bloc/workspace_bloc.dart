/// Workspace BLoC — manages the main app state.
///
/// Inspired by AppFlowy's TabsBloc + HomeBloc pattern.
library;

import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

import '../api/flowforge_api.dart';

part 'workspace_bloc.freezed.dart';

// ── Events ──

@freezed
class WorkspaceEvent with _$WorkspaceEvent {
  /// Load all workflows from the server.
  const factory WorkspaceEvent.loadWorkflows() = _LoadWorkflows;

  /// Select a workflow to edit.
  const factory WorkspaceEvent.selectWorkflow(String workflowId) =
      _SelectWorkflow;

  /// Clear the selection (go back to dashboard).
  const factory WorkspaceEvent.clearSelection() = _ClearSelection;

  /// Switch the active page (dashboard, editor, settings).
  const factory WorkspaceEvent.switchPage(int pageIndex) = _SwitchPage;
}

// ── State ──

@freezed
class WorkspaceState with _$WorkspaceState {
  const factory WorkspaceState({
    @Default([]) List<Workflow> workflows,
    @Default(0) int selectedPageIndex,
    String? selectedWorkflowId,
    @Default(false) bool loading,
    String? error,
  }) = _WorkspaceState;
}

// ── BLoC ──

class WorkspaceBloc extends Bloc<WorkspaceEvent, WorkspaceState> {
  final FlowForgeApi api;

  WorkspaceBloc({required this.api}) : super(const WorkspaceState()) {
    on<_LoadWorkflows>(_onLoadWorkflows);
    on<_SelectWorkflow>(_onSelectWorkflow);
    on<_ClearSelection>(_onClearSelection);
    on<_SwitchPage>(_onSwitchPage);
  }

  Future<void> _onLoadWorkflows(
    _LoadWorkflows event,
    Emitter<WorkspaceState> emit,
  ) async {
    emit(state.copyWith(loading: true, error: null));
    try {
      final workflows = await api.listWorkflows();
      emit(state.copyWith(workflows: workflows, loading: false));
    } catch (e) {
      emit(state.copyWith(loading: false, error: e.toString()));
    }
  }

  void _onSelectWorkflow(
    _SelectWorkflow event,
    Emitter<WorkspaceState> emit,
  ) {
    emit(state.copyWith(
      selectedWorkflowId: event.workflowId,
      selectedPageIndex: 1, // switch to editor
    ));
  }

  void _onClearSelection(
    _ClearSelection event,
    Emitter<WorkspaceState> emit,
  ) {
    emit(state.copyWith(
      selectedWorkflowId: null,
      selectedPageIndex: 0, // back to dashboard
    ));
  }

  void _onSwitchPage(
    _SwitchPage event,
    Emitter<WorkspaceState> emit,
  ) {
    emit(state.copyWith(selectedPageIndex: event.pageIndex));
  }
}
