import React from "react";
import { useAppSelector, useAppDispatch } from "../../store";
import { setActivePanel, hideSearchPanel } from "../../store/slices/panelSlice";
import FilePanel from "../panels/FilePanel";
import { SearchPanel } from "../panels/SearchPanel";
import "./MultiPanelLayout.css";

const MultiPanelLayout: React.FC = () => {
  const dispatch = useAppDispatch();
  const { panels, activePanelId, gridLayout, panelOrder, searchPanelVisible } = useAppSelector(
    (state) => state.panels
  );

  const handlePanelClick = (panelId: string) => {
    dispatch(setActivePanel(panelId));
  };

  const handleCloseSearchPanel = () => {
    dispatch(hideSearchPanel());
  };

  const gridStyle = {
    display: "grid",
    gridTemplateRows: `repeat(${gridLayout.rows}, 1fr)`,
    gridTemplateColumns: `repeat(${gridLayout.cols}, 1fr)`,
    gap: "1px",
    height: "100%",
    width: "100%",
  };

  const visiblePanels = panelOrder.slice(0, gridLayout.rows * gridLayout.cols);

  return (
    <div className="multi-panel-layout" data-testid="multi-panel-layout">
      <div className={`layout-content ${searchPanelVisible ? 'with-search-panel' : ''}`}>
        <div className="grid-container" style={gridStyle} role="grid">
          {visiblePanels.map((panelId) => {
            const panel = panels[panelId];
            if (!panel) return null;

            const isActive = activePanelId === panelId;

            return (
              <div
                key={panelId}
                className={`panel-wrapper ${isActive ? "active" : ""}`}
                data-testid={`panel-wrapper-${panelId}`}
                onClick={() => handlePanelClick(panelId)}
              >
                <FilePanel panel={panel} isActive={isActive} />
              </div>
            );
          })}
        </div>
        
        {/* Search Panel Overlay */}
        {searchPanelVisible && (
          <div className="search-panel-overlay">
            <SearchPanel
              isActive={true}
              onClose={handleCloseSearchPanel}
            />
          </div>
        )}
      </div>
    </div>
  );
};

export default MultiPanelLayout;
