import React, { useEffect, useState } from 'react';
import { useAppSelector, useAppDispatch } from '../../store';
import { setActivePanel } from '../../store/slices/panelSlice';
import FilePanel from '../panels/FilePanel';
import './MultiPanelLayout.css';

const MultiPanelLayout: React.FC = () => {
  const dispatch = useAppDispatch();
  const { panels, activePanelId, gridLayout, panelOrder } = useAppSelector(state => state.panels);
  const [addressBarActive, setAddressBarActive] = useState(false);

  const handlePanelClick = (panelId: string) => {
    dispatch(setActivePanel(panelId));
  };

  // Global keyboard shortcut handler - only affects the active panel
  useEffect(() => {
    const handleGlobalKeyDown = (event: KeyboardEvent) => {
      // Only handle shortcuts if we have an active panel
      if (!activePanelId || !panels[activePanelId]) return;

      // Handle Ctrl+L for address bar focus
      if ((event.ctrlKey || event.metaKey) && event.key === 'l') {
        event.preventDefault();
        setAddressBarActive(true);
        return;
      }

      // Add other global shortcuts here in the future
      // Examples: Ctrl+N (new tab), Ctrl+W (close tab), etc.
    };

    document.addEventListener('keydown', handleGlobalKeyDown);
    return () => document.removeEventListener('keydown', handleGlobalKeyDown);
  }, [activePanelId, panels]);

  const gridStyle = {
    display: 'grid',
    gridTemplateRows: `repeat(${gridLayout.rows}, 1fr)`,
    gridTemplateColumns: `repeat(${gridLayout.cols}, 1fr)`,
    gap: '1px',
    height: '100%',
    width: '100%',
  };

  const visiblePanels = panelOrder.slice(0, gridLayout.rows * gridLayout.cols);

  return (
    <div className="multi-panel-layout">
      <div className="grid-container" style={gridStyle}>
        {visiblePanels.map((panelId) => {
          const panel = panels[panelId];
          if (!panel) return null;

          const isActive = activePanelId === panelId;

          return (
            <div
              key={panelId}
              className={`panel-wrapper ${isActive ? 'active' : ''}`}
              onClick={() => handlePanelClick(panelId)}
            >
              <FilePanel 
                panel={panel} 
                isActive={isActive}
                addressBarActive={isActive ? addressBarActive : false}
                onAddressBarFocus={() => setAddressBarActive(false)}
              />
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default MultiPanelLayout;