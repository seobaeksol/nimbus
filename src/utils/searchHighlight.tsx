/**
 * Search Result Highlighting Utilities
 * 
 * Provides functions for highlighting search matches in file names, content,
 * and other text fields with support for multiple match types and patterns.
 */

import React from 'react';
import { ContentMatch, MatchType } from '@/types';

export interface HighlightOptions {
  className?: string;
  caseSensitive?: boolean;
  maxLength?: number;
  showLineNumbers?: boolean;
}

/**
 * Highlights text based on search patterns with support for multiple match types
 */
export function highlightText(
  text: string,
  searchTerm: string,
  matchType: MatchType = 'exact_name',
  options: HighlightOptions = {}
): React.ReactNode {
  if (!text || !searchTerm) {
    return text;
  }

  const {
    className = 'search-match-highlight',
    caseSensitive = false,
    maxLength
  } = options;

  // Truncate text if needed
  const displayText = maxLength && text.length > maxLength
    ? text.substring(0, maxLength) + '...'
    : text;

  let pattern: RegExp;

  try {
    switch (matchType) {
      case 'exact_name':
        // Exact match highlighting
        pattern = new RegExp(
          `(${escapeRegExp(searchTerm)})`,
          caseSensitive ? 'g' : 'gi'
        );
        break;

      case 'fuzzy_name':
        // Fuzzy matching - highlight individual characters
        return highlightFuzzyMatch(displayText, searchTerm, className, caseSensitive);

      case 'content':
        // Content search - might be regex
        pattern = new RegExp(
          `(${escapeRegExp(searchTerm)})`,
          caseSensitive ? 'g' : 'gi'
        );
        break;

      case 'extension':
        // Extension matching
        pattern = new RegExp(
          `\\.(${escapeRegExp(searchTerm)})$`,
          caseSensitive ? 'g' : 'gi'
        );
        break;

      case 'directory':
        // Directory path matching
        pattern = new RegExp(
          `(${escapeRegExp(searchTerm)})`,
          caseSensitive ? 'g' : 'gi'
        );
        break;

      default:
        pattern = new RegExp(
          `(${escapeRegExp(searchTerm)})`,
          caseSensitive ? 'g' : 'gi'
        );
    }

    const parts = displayText.split(pattern);
    
    return (
      <>
        {parts.map((part, index) => {
          const isMatch = pattern.test(part);
          pattern.lastIndex = 0; // Reset regex state
          
          if (isMatch) {
            return (
              <span key={index} className={className}>
                {part}
              </span>
            );
          }
          return part;
        })}
      </>
    );
  } catch (error) {
    // Fallback to plain text if regex fails
    console.warn('Search highlighting failed:', error);
    return displayText;
  }
}

/**
 * Highlights content matches with precise positioning
 */
export function highlightContentMatches(
  text: string,
  matches: ContentMatch[],
  options: HighlightOptions = {}
): React.ReactNode {
  if (!text || !matches || matches.length === 0) {
    return text;
  }

  const { className = 'search-match-highlight', maxLength } = options;

  // Sort matches by start position to ensure proper highlighting
  const sortedMatches = [...matches].sort((a, b) => a.matchStart - b.matchStart);

  // Check for overlapping matches and merge them
  const mergedMatches = mergeOverlappingMatches(sortedMatches);

  // Truncate text if needed
  const displayText = maxLength && text.length > maxLength
    ? text.substring(0, maxLength) + '...'
    : text;

  const parts: React.ReactNode[] = [];
  let lastIndex = 0;

  mergedMatches.forEach((match, index) => {
    const { matchStart, matchEnd } = match;

    // Ensure match positions are within bounds
    const safeStart = Math.max(0, Math.min(matchStart, displayText.length));
    const safeEnd = Math.max(safeStart, Math.min(matchEnd, displayText.length));

    // Add text before match
    if (safeStart > lastIndex) {
      parts.push(displayText.substring(lastIndex, safeStart));
    }

    // Add highlighted match
    if (safeEnd > safeStart) {
      parts.push(
        <span key={index} className={className}>
          {displayText.substring(safeStart, safeEnd)}
        </span>
      );
    }

    lastIndex = safeEnd;
  });

  // Add remaining text
  if (lastIndex < displayText.length) {
    parts.push(displayText.substring(lastIndex));
  }

  return <>{parts}</>;
}

/**
 * Highlights line content with line number display
 */
export function highlightLineContent(
  match: ContentMatch,
  options: HighlightOptions = {}
): React.ReactNode {
  const {
    className = 'search-match-highlight',
    showLineNumbers = true,
    maxLength = 200
  } = options;

  const { lineNumber, lineContent, matchStart, matchEnd } = match;

  // Trim and truncate line content
  const trimmedContent = lineContent.trim();
  const displayContent = maxLength && trimmedContent.length > maxLength
    ? trimmedContent.substring(0, maxLength) + '...'
    : trimmedContent;

  // Adjust match positions for trimmed content
  const leadingWhitespace = lineContent.length - lineContent.trimStart().length;
  const adjustedStart = Math.max(0, matchStart - leadingWhitespace);
  const adjustedEnd = Math.max(adjustedStart, matchEnd - leadingWhitespace);

  return (
    <div className="match-line">
      {showLineNumbers && (
        <span className="line-number">
          Line {lineNumber}:
        </span>
      )}
      <span className="line-content">
        {highlightContentMatches(displayContent, [
          { ...match, matchStart: adjustedStart, matchEnd: adjustedEnd }
        ], { className })}
      </span>
    </div>
  );
}

/**
 * Highlights fuzzy matches character by character
 */
function highlightFuzzyMatch(
  text: string,
  searchTerm: string,
  className: string,
  caseSensitive: boolean
): React.ReactNode {
  const searchChars = (caseSensitive ? searchTerm : searchTerm.toLowerCase()).split('');
  const textChars = (caseSensitive ? text : text.toLowerCase()).split('');
  const originalChars = text.split('');

  const result: React.ReactNode[] = [];
  let searchIndex = 0;

  originalChars.forEach((char, index) => {
    if (searchIndex < searchChars.length && textChars[index] === searchChars[searchIndex]) {
      result.push(
        <span key={index} className={className}>
          {char}
        </span>
      );
      searchIndex++;
    } else {
      result.push(char);
    }
  });

  return <>{result}</>;
}

/**
 * Merges overlapping content matches to prevent nested highlighting
 */
function mergeOverlappingMatches(matches: ContentMatch[]): ContentMatch[] {
  if (matches.length <= 1) return matches;

  const merged: ContentMatch[] = [];
  let current = { ...matches[0] };

  for (let i = 1; i < matches.length; i++) {
    const next = matches[i];

    // Check if current and next overlap or are adjacent
    if (next.matchStart <= current.matchEnd) {
      // Merge the matches
      current.matchEnd = Math.max(current.matchEnd, next.matchEnd);
    } else {
      // No overlap, add current to result and start new one
      merged.push(current);
      current = { ...next };
    }
  }

  merged.push(current);
  return merged;
}

/**
 * Escapes special regex characters
 */
function escapeRegExp(string: string): string {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * Creates context around a match for better readability
 */
export function createMatchContext(
  text: string,
  matchStart: number,
  matchEnd: number,
  contextLength: number = 50
): { start: number; end: number; prefix: string; suffix: string } {
  const start = Math.max(0, matchStart - contextLength);
  const end = Math.min(text.length, matchEnd + contextLength);

  const prefix = start > 0 ? '...' : '';
  const suffix = end < text.length ? '...' : '';

  return {
    start,
    end,
    prefix,
    suffix
  };
}

/**
 * Highlights search terms in file paths with directory separation
 */
export function highlightFilePath(
  path: string,
  searchTerm: string,
  options: HighlightOptions = {}
): React.ReactNode {
  if (!path || !searchTerm) {
    return path;
  }

  const { className = 'search-match-highlight' } = options;
  const pathParts = path.split('/');

  return (
    <>
      {pathParts.map((part, index) => (
        <React.Fragment key={index}>
          {index > 0 && <span className="path-separator">/</span>}
          {highlightText(part, searchTerm, 'exact_name', { className })}
        </React.Fragment>
      ))}
    </>
  );
}