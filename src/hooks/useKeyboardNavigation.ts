/**
 * Keyboard Navigation Hook for Search Results
 * 
 * Provides comprehensive keyboard navigation for search results with support for
 * arrow keys, selection, quick actions, and accessibility.
 */

import { useCallback, useEffect, useState, useRef } from 'react';
import { SearchResult } from '@/types';

export interface KeyboardNavigationOptions {
  onResultSelect?: (result: SearchResult, index: number) => void;
  onResultActivate?: (result: SearchResult, index: number) => void;
  onEscape?: () => void;
  onSearch?: () => void;
  enableQuickActions?: boolean;
  enableTypeahead?: boolean;
  wrapNavigation?: boolean;
}

export interface KeyboardNavigationState {
  selectedIndex: number;
  isNavigating: boolean;
  typeaheadBuffer: string;
  lastKeyTime: number;
}

export function useKeyboardNavigation(
  results: SearchResult[],
  options: KeyboardNavigationOptions = {}
) {
  const {
    onResultSelect,
    onResultActivate,
    onEscape,
    onSearch,
    enableQuickActions = true,
    enableTypeahead = true,
    wrapNavigation = true
  } = options;

  const [state, setState] = useState<KeyboardNavigationState>({
    selectedIndex: -1,
    isNavigating: false,
    typeaheadBuffer: '',
    lastKeyTime: 0
  });

  const containerRef = useRef<HTMLElement>(null);
  const typeaheadTimeoutRef = useRef<NodeJS.Timeout>();

  // Clear typeahead buffer after delay
  const clearTypeahead = useCallback(() => {
    if (typeaheadTimeoutRef.current) {
      clearTimeout(typeaheadTimeoutRef.current);
    }
    typeaheadTimeoutRef.current = setTimeout(() => {
      setState(prev => ({ ...prev, typeaheadBuffer: '' }));
    }, 1000);
  }, []);

  // Find result by typeahead matching
  const findByTypeahead = useCallback((buffer: string) => {
    const lowerBuffer = buffer.toLowerCase();
    return results.findIndex(result => 
      result.name.toLowerCase().startsWith(lowerBuffer)
    );
  }, [results]);

  // Navigate to specific index
  const navigateToIndex = useCallback((newIndex: number) => {
    if (results.length === 0) return;

    let targetIndex = newIndex;

    if (wrapNavigation) {
      if (targetIndex < 0) {
        targetIndex = results.length - 1;
      } else if (targetIndex >= results.length) {
        targetIndex = 0;
      }
    } else {
      targetIndex = Math.max(0, Math.min(targetIndex, results.length - 1));
    }

    setState(prev => ({
      ...prev,
      selectedIndex: targetIndex,
      isNavigating: true
    }));

    const selectedResult = results[targetIndex];
    if (selectedResult && onResultSelect) {
      onResultSelect(selectedResult, targetIndex);
    }

    // Scroll to selected item
    const selectedElement = containerRef.current?.querySelector(
      `[data-result-index="${targetIndex}"]`
    ) as HTMLElement;

    if (selectedElement) {
      selectedElement.scrollIntoView({
        behavior: 'smooth',
        block: 'nearest'
      });
      // Focus the element for accessibility
      selectedElement.focus();
    }
  }, [results, wrapNavigation, onResultSelect]);

  // Handle keyboard events
  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    const { key, ctrlKey, metaKey, shiftKey, altKey } = event;
    const modifierPressed = ctrlKey || metaKey || shiftKey || altKey;

    // Ignore keyboard events when typing in input fields
    const activeElement = document.activeElement;
    if (activeElement?.tagName === 'INPUT' || activeElement?.tagName === 'TEXTAREA') {
      // Allow Escape to exit input fields
      if (key === 'Escape') {
        (activeElement as HTMLElement).blur();
        onEscape?.();
      }
      return;
    }

    switch (key) {
      case 'ArrowDown':
        event.preventDefault();
        navigateToIndex(state.selectedIndex + 1);
        break;

      case 'ArrowUp':
        event.preventDefault();
        navigateToIndex(state.selectedIndex - 1);
        break;

      case 'Home':
        if (results.length > 0) {
          event.preventDefault();
          navigateToIndex(0);
        }
        break;

      case 'End':
        if (results.length > 0) {
          event.preventDefault();
          navigateToIndex(results.length - 1);
        }
        break;

      case 'PageDown':
        event.preventDefault();
        navigateToIndex(state.selectedIndex + 10);
        break;

      case 'PageUp':
        event.preventDefault();
        navigateToIndex(state.selectedIndex - 10);
        break;

      case 'Enter':
        if (state.selectedIndex >= 0 && state.selectedIndex < results.length) {
          event.preventDefault();
          const selectedResult = results[state.selectedIndex];
          onResultActivate?.(selectedResult, state.selectedIndex);
        }
        break;

      case 'Escape':
        event.preventDefault();
        setState(prev => ({ ...prev, selectedIndex: -1, isNavigating: false }));
        onEscape?.();
        break;

      case ' ': // Spacebar
        if (state.selectedIndex >= 0 && state.selectedIndex < results.length) {
          event.preventDefault();
          const selectedResult = results[state.selectedIndex];
          onResultSelect?.(selectedResult, state.selectedIndex);
        }
        break;

      // Quick actions (when enabled)
      case 'o':
        if (enableQuickActions && !modifierPressed && state.selectedIndex >= 0) {
          event.preventDefault();
          const selectedResult = results[state.selectedIndex];
          onResultActivate?.(selectedResult, state.selectedIndex);
        }
        break;

      case 'c':
        if (enableQuickActions && (ctrlKey || metaKey) && state.selectedIndex >= 0) {
          event.preventDefault();
          const selectedResult = results[state.selectedIndex];
          // Copy file path to clipboard
          navigator.clipboard?.writeText(selectedResult.path);
        }
        break;

      case 'f':
        if (enableQuickActions && (ctrlKey || metaKey)) {
          event.preventDefault();
          onSearch?.();
        }
        break;

      default:
        // Typeahead navigation
        if (enableTypeahead && !modifierPressed && key.length === 1 && /[a-zA-Z0-9]/.test(key)) {
          const currentTime = Date.now();
          let newBuffer = state.typeaheadBuffer;

          // Reset buffer if too much time has passed
          if (currentTime - state.lastKeyTime > 1000) {
            newBuffer = '';
          }

          newBuffer += key.toLowerCase();
          
          setState(prev => ({
            ...prev,
            typeaheadBuffer: newBuffer,
            lastKeyTime: currentTime
          }));

          // Find matching result
          const matchIndex = findByTypeahead(newBuffer);
          if (matchIndex >= 0) {
            navigateToIndex(matchIndex);
          }

          clearTypeahead();
        }
        break;
    }
  }, [
    state.selectedIndex,
    state.typeaheadBuffer,
    state.lastKeyTime,
    results,
    navigateToIndex,
    findByTypeahead,
    clearTypeahead,
    onResultSelect,
    onResultActivate,
    onEscape,
    onSearch,
    enableQuickActions,
    enableTypeahead
  ]);

  // Attach/detach keyboard listeners
  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
      if (typeaheadTimeoutRef.current) {
        clearTimeout(typeaheadTimeoutRef.current);
      }
    };
  }, [handleKeyDown]);

  // Reset selection when results change
  useEffect(() => {
    setState(prev => ({
      ...prev,
      selectedIndex: results.length > 0 ? Math.min(prev.selectedIndex, results.length - 1) : -1
    }));
  }, [results]);

  // Get current selected result
  const selectedResult = state.selectedIndex >= 0 && state.selectedIndex < results.length
    ? results[state.selectedIndex]
    : null;

  // Focus management
  const focusResult = useCallback((index: number) => {
    const element = containerRef.current?.querySelector(
      `[data-result-index="${index}"]`
    ) as HTMLElement;
    element?.focus();
  }, []);

  // Programmatically set selection
  const setSelectedIndex = useCallback((index: number) => {
    navigateToIndex(index);
  }, [navigateToIndex]);

  // Clear selection
  const clearSelection = useCallback(() => {
    setState(prev => ({ ...prev, selectedIndex: -1, isNavigating: false }));
  }, []);

  return {
    // State
    selectedIndex: state.selectedIndex,
    selectedResult,
    isNavigating: state.isNavigating,
    typeaheadBuffer: state.typeaheadBuffer,

    // Actions
    setSelectedIndex,
    clearSelection,
    focusResult,
    navigateToIndex,

    // Refs
    containerRef,

    // Keyboard shortcuts info for help display
    shortcuts: {
      navigation: [
        { key: '↑/↓', description: 'Navigate results' },
        { key: 'Home/End', description: 'First/last result' },
        { key: 'Page Up/Down', description: 'Jump 10 results' }
      ],
      actions: [
        { key: 'Enter', description: 'Open selected result' },
        { key: 'Space', description: 'Select result' },
        { key: 'Escape', description: 'Clear selection' }
      ],
      quickActions: enableQuickActions ? [
        { key: 'O', description: 'Open result' },
        { key: 'Ctrl+C', description: 'Copy path' },
        { key: 'Ctrl+F', description: 'Focus search' }
      ] : [],
      typeahead: enableTypeahead ? [
        { key: 'Type', description: 'Quick find by name' }
      ] : []
    }
  };
}