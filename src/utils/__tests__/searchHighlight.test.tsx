/**
 * Tests for Search Highlighting Utilities
 */

import React from 'react';
import { render } from '@testing-library/react';
import { highlightText, highlightContentMatches, highlightLineContent } from '../searchHighlight';
import { ContentMatch } from '@/types';

describe('Search Highlighting Utilities', () => {
  describe('highlightText', () => {
    it('should highlight exact matches', () => {
      const result = highlightText('Hello World', 'World', 'exact_name');
      const { container } = render(<div>{result}</div>);
      
      expect(container.textContent).toBe('Hello World');
      expect(container.querySelector('.search-match-highlight')).toBeTruthy();
    });

    it('should handle case insensitive matches', () => {
      const result = highlightText('Hello WORLD', 'world', 'exact_name', { caseSensitive: false });
      const { container } = render(<div>{result}</div>);
      
      expect(container.textContent).toBe('Hello WORLD');
      expect(container.querySelector('.search-match-highlight')).toBeTruthy();
    });

    it('should return original text when no match', () => {
      const result = highlightText('Hello World', 'xyz', 'exact_name');
      const { container } = render(<div>{result}</div>);
      
      expect(container.textContent).toBe('Hello World');
      expect(container.querySelector('.search-match-highlight')).toBeNull();
    });
  });

  describe('highlightContentMatches', () => {
    it('should highlight content matches with positions', () => {
      const matches: ContentMatch[] = [
        {
          lineNumber: 1,
          lineContent: 'This is a test line',
          matchStart: 10,
          matchEnd: 14
        }
      ];

      const result = highlightContentMatches('This is a test line', matches);
      const { container } = render(<div>{result}</div>);
      
      expect(container.textContent).toBe('This is a test line');
      expect(container.querySelector('.search-match-highlight')).toBeTruthy();
    });

    it('should handle multiple matches', () => {
      const matches: ContentMatch[] = [
        {
          lineNumber: 1,
          lineContent: 'test test test',
          matchStart: 0,
          matchEnd: 4
        },
        {
          lineNumber: 1,
          lineContent: 'test test test',
          matchStart: 5,
          matchEnd: 9
        }
      ];

      const result = highlightContentMatches('test test test', matches);
      const { container } = render(<div>{result}</div>);
      
      expect(container.textContent).toBe('test test test');
      expect(container.querySelectorAll('.search-match-highlight')).toHaveLength(2);
    });
  });

  describe('highlightLineContent', () => {
    it('should render line content with line number', () => {
      const match: ContentMatch = {
        lineNumber: 42,
        lineContent: 'This is a test line',
        matchStart: 10,
        matchEnd: 14
      };

      const result = highlightLineContent(match);
      const { container } = render(<div>{result}</div>);
      
      expect(container.textContent).toContain('Line 42:');
      expect(container.textContent).toContain('This is a test line');
      expect(container.querySelector('.line-number')).toBeTruthy();
      expect(container.querySelector('.line-content')).toBeTruthy();
    });
  });
});