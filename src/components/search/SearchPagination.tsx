/**
 * Search Pagination Component
 * 
 * Provides pagination controls for search results with page size options
 * and navigation controls.
 */

import React, { useCallback } from 'react';
import { useAppDispatch } from '@/store';
import { setSearchPage, setPageSize } from '@/store/slices/searchSlice';

interface SearchPaginationProps {
  searchId: string;
  currentPage: number;
  pageSize: number;
  totalPages: number;
  totalResults: number;
  className?: string;
}

export const SearchPagination: React.FC<SearchPaginationProps> = ({
  searchId,
  currentPage,
  pageSize,
  totalPages,
  totalResults,
  className = ''
}) => {
  const dispatch = useAppDispatch();

  const handlePageChange = useCallback((newPage: number) => {
    if (newPage >= 0 && newPage < totalPages) {
      dispatch(setSearchPage({ searchId, page: newPage }));
    }
  }, [dispatch, searchId, totalPages]);

  const handlePageSizeChange = useCallback((event: React.ChangeEvent<HTMLSelectElement>) => {
    const newPageSize = parseInt(event.target.value);
    dispatch(setPageSize({ searchId, pageSize: newPageSize }));
  }, [dispatch, searchId]);

  const getPageNumbers = useCallback((): number[] => {
    const pages: number[] = [];
    const maxVisible = 7; // Maximum number of page buttons to show
    
    if (totalPages <= maxVisible) {
      // Show all pages if total is small
      for (let i = 0; i < totalPages; i++) {
        pages.push(i);
      }
    } else {
      // Show first page
      pages.push(0);
      
      // Calculate range around current page
      let start = Math.max(1, currentPage - 2);
      let end = Math.min(totalPages - 2, currentPage + 2);
      
      // Add ellipsis if needed
      if (start > 1) {
        pages.push(-1); // -1 represents ellipsis
      }
      
      // Add pages around current
      for (let i = start; i <= end; i++) {
        if (i > 0 && i < totalPages - 1) {
          pages.push(i);
        }
      }
      
      // Add ellipsis if needed
      if (end < totalPages - 2) {
        pages.push(-1); // -1 represents ellipsis
      }
      
      // Show last page
      if (totalPages > 1) {
        pages.push(totalPages - 1);
      }
    }
    
    return pages;
  }, [currentPage, totalPages]);

  if (totalPages <= 1) {
    return null; // Don't show pagination for single page
  }

  const startResult = currentPage * pageSize + 1;
  const endResult = Math.min((currentPage + 1) * pageSize, totalResults);

  return (
    <div className={`search-pagination ${className}`}>
      {/* Results summary */}
      <div className="pagination-info">
        <span className="results-range">
          {startResult}-{endResult} of {totalResults} results
        </span>
        
        <div className="page-size-control">
          <label htmlFor="page-size-select">Show:</label>
          <select
            id="page-size-select"
            value={pageSize}
            onChange={handlePageSizeChange}
            className="page-size-select"
          >
            <option value={25}>25</option>
            <option value={50}>50</option>
            <option value={100}>100</option>
            <option value={200}>200</option>
          </select>
          <span>per page</span>
        </div>
      </div>

      {/* Navigation controls */}
      <div className="pagination-controls">
        <button
          className="page-btn prev-btn"
          onClick={() => handlePageChange(currentPage - 1)}
          disabled={currentPage === 0}
          title="Previous page"
        >
          ←
        </button>

        {getPageNumbers().map((pageNum, index) => (
          <React.Fragment key={`page-${pageNum}-${index}`}>
            {pageNum === -1 ? (
              <span className="page-ellipsis">...</span>
            ) : (
              <button
                className={`page-btn ${pageNum === currentPage ? 'active' : ''}`}
                onClick={() => handlePageChange(pageNum)}
              >
                {pageNum + 1}
              </button>
            )}
          </React.Fragment>
        ))}

        <button
          className="page-btn next-btn"
          onClick={() => handlePageChange(currentPage + 1)}
          disabled={currentPage === totalPages - 1}
          title="Next page"
        >
          →
        </button>
      </div>

      {/* Page info */}
      <div className="page-info">
        Page {currentPage + 1} of {totalPages}
      </div>
    </div>
  );
};