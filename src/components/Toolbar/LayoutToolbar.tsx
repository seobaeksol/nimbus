import React, { useEffect } from 'react';
import { useAppSelector, useAppDispatch } from '../../store';
import { setGridLayout } from '../../store/slices/panelSlice';
import './LayoutToolbar.css';

const LayoutToolbar: React.FC = () => {
  const dispatch = useAppDispatch();
  const { gridLayout, presetLayouts } = useAppSelector(state => state.panels);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Ctrl+G shortcuts for grid layouts
      if (event.ctrlKey && event.key.toLowerCase() === 'g') {
        event.preventDefault();
        
        // Listen for the next key
        const handleLayoutKey = (e: KeyboardEvent) => {
          e.preventDefault();
          window.removeEventListener('keydown', handleLayoutKey);
          
          const layouts = {
            '1': presetLayouts.find(l => l.cols === 1 && l.rows === 1), // Single
            '2': presetLayouts.find(l => l.cols === 2 && l.rows === 1), // Dual
            '3': presetLayouts.find(l => l.cols === 2 && l.rows === 2), // 2x2
            '4': presetLayouts.find(l => l.cols === 3 && l.rows === 2), // 2x3
            '5': presetLayouts.find(l => l.cols === 2 && l.rows === 3), // 3x2
          };
          
          const selectedLayout = layouts[e.key as keyof typeof layouts];
          if (selectedLayout) {
            dispatch(setGridLayout(selectedLayout));
          }
        };
        
        window.addEventListener('keydown', handleLayoutKey);
        
        // Remove listener after 2 seconds if no key is pressed
        setTimeout(() => {
          window.removeEventListener('keydown', handleLayoutKey);
        }, 2000);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [dispatch, presetLayouts]);

  const handleLayoutChange = (layout: typeof gridLayout) => {
    dispatch(setGridLayout(layout));
  };

  const getLayoutIcon = (rows: number, cols: number) => {
    return `${rows}×${cols}`;
  };

  return (
    <div className="layout-toolbar">
      <div className="toolbar-section">
        <span className="toolbar-label">Grid Layout:</span>
        <div className="layout-buttons">
          {presetLayouts.map((layout) => (
            <button
              key={`${layout.rows}x${layout.cols}`}
              className={`layout-button ${
                gridLayout.rows === layout.rows && gridLayout.cols === layout.cols 
                  ? 'active' 
                  : ''
              }`}
              onClick={() => handleLayoutChange(layout)}
              title={`${layout.name} (Ctrl+G+${
                layout.cols === 1 && layout.rows === 1 ? '1' :
                layout.cols === 2 && layout.rows === 1 ? '2' :
                layout.cols === 2 && layout.rows === 2 ? '3' :
                layout.cols === 3 && layout.rows === 2 ? '4' :
                layout.cols === 2 && layout.rows === 3 ? '5' : ''
              })`}
            >
              <span className="layout-icon">
                {getLayoutIcon(layout.rows, layout.cols)}
              </span>
              <span className="layout-name">{layout.name}</span>
            </button>
          ))}
        </div>
      </div>
      
      <div className="toolbar-section">
        <span className="current-layout">
          Active: {gridLayout.name} ({gridLayout.rows * gridLayout.cols} panels)
        </span>
      </div>
      
      <div className="keyboard-hint">
        Press <kbd>Ctrl</kbd>+<kbd>G</kbd> then <kbd>1-5</kbd> for quick layout switching
      </div>
    </div>
  );
};

export default LayoutToolbar;