import { SearchOptions } from 'src/components/models';
import { nodeConnectors, nodeNameAsString, NodeSummary } from 'src/util';

// By default, search will be case insensitive and return all results
const defaultSearchOptions: SearchOptions = {
  caseSensitive: false,
  numResults: undefined,
};

// jettySearch takes an array of items of type T, a mapper that maps T -> string, a search term,
// and, optionally, a SearchOptions object.
//
// The term is split on spaces (unless double-quoted), and then each item is checked to see if it includes
// all of the term components.
//
// If a value is provided for numResults, the search will exit early once that number has been found.
// This works well because there is no scoring going on.
//
//The results are returned in the order the items were provided.
export const jettySearch = <T>(
  items: T[],
  itemMapper: (item: T) => string,
  term: string,
  options: SearchOptions = {}
): T[] => {
  options = { ...defaultSearchOptions, ...options };

  const terms =
    term.match(/(".*?"|[^"\s]+)(?=\s*|\s*$)/g) ??
    []
      // if a term is surrounded by quotes, strip them
      .map((t) => {
        if (
          t.length > 1 &&
          t.charAt(0) == '"' &&
          t.charAt(t.length - 1) == '"'
        ) {
          return t.replaceAll('"', '');
        } else {
          return t;
        }
      });

  let result: T[] = [];

  if (options.numResults !== undefined) {
    items.find((i) => {
      const targetString = itemMapper(i);
      if (
        (options.caseSensitive &&
          terms.every((t) => targetString.includes(t))) ||
        terms.every((t) =>
          targetString.toLocaleLowerCase().includes(t.toLocaleLowerCase())
        )
      ) {
        result.push(i);
        if (result.length == options.numResults) {
          return true;
        } else {
          return false;
        }
      }
    });
  } else {
    result = items.filter((i) => {
      const targetString = itemMapper(i);
      return (
        (options.caseSensitive &&
          terms.every((t) => targetString.includes(t))) ||
        terms.every((t) =>
          targetString.toLocaleLowerCase().includes(t.toLocaleLowerCase())
        )
      );
    });
  }

  return result;
};

// Given a node summary, map it to a string that can be used for search
export const mapNodeSummaryforSearch = (summary: NodeSummary): string => {
  if ('Tag' in summary) {
    return [
      nodeNameAsString(summary),
      summary.Tag.description,
      ...nodeConnectors(summary),
    ].join(' ');
  } else if ('Asset' in summary) {
    return [
      nodeNameAsString(summary),
      summary.Asset.asset_type,
      ...nodeConnectors(summary),
    ].join(' ');
  } else {
    return [nodeNameAsString(summary), ...nodeConnectors(summary)].join(' ');
  }
};
