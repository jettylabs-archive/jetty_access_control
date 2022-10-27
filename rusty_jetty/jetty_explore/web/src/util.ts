import { Result } from 'postcss';
import { exportFile } from 'quasar';
import {
  AssetSummary,
  GroupName,
  GroupSummary,
  NodePath,
  SearchOptions,
  TagSummary,
  UserName,
  UserSummary,
} from './components/models';

export const getBadgeColor = (stringInput: string) => {
  if (stringInput.toLocaleLowerCase() == 'jetty') {
    return '#f47124';
  }

  // Credit to https://stackoverflow.com/a/66494926
  const stringUniqueHash = [...stringInput].reduce((acc, char) => {
    return char.charCodeAt(0) + ((acc << 5) - acc);
  }, 0);
  return `hsl(${stringUniqueHash % 360}, 95%, 35%)`;
};

export const getNodeIcon = (stringInput: string) => {
  let icon = 'person';
  switch (stringInput) {
    case 'user':
      icon = 'person';
      break;
    case 'asset':
      icon = 'table_chart';
      break;
    case 'group':
      icon = 'group';
      break;
    case 'tag':
      icon = 'sell';
      break;
  }
  return icon;
};

export function wrapCsvValue(val) {
  let formatted = val === void 0 || val === null ? '' : String(val);

  formatted = formatted.split('"').join('""');
  /**
   * Excel accepts \n and \r in strings, but some other CSV parsers do not
   * Uncomment the next two lines to escape new lines
   */
  // .split('\n').join('\\n')
  // .split('\r').join('\\r')

  return `"${formatted}"`;
}

export function downloadCSV(filename, columns, rows) {
  // naive encoding to csv format
  const content = [columns.map((c) => wrapCsvValue(c))]
    .concat(rows.map((row) => row.map((val) => wrapCsvValue(val)).join(',')))
    .join('\r\n');

  const status = exportFile(filename, content, 'text/csv');

  if (status !== true) {
    console.log('Browser denied file download...');
  }
}

export function fetchJson(path: string) {
  const requestOptions: RequestInit = {
    method: 'GET',
    redirect: 'follow',
  };

  return fetch(path, requestOptions)
    .then((response) => response.json())
    .catch((error) => console.log('error fetching data:', error));
}

export function nodeNameAsString(
  node: GroupSummary | UserSummary | AssetSummary | TagSummary
): string {
  if ('Group' in node) {
    return node.Group.name.Group.origin + '::' + node.Group.name.Group.name;
  } else if ('User' in node) {
    return node.User.name.User;
  } else if ('Asset' in node) {
    return node.Asset.name.Asset.uri;
  } else if ('Tag' in node) {
    return node.Tag.name.Tag;
  }
}

export function nodeConnectors(
  node: GroupSummary | UserSummary | AssetSummary | TagSummary
): string[] {
  if ('Group' in node) {
    return node.Group.connectors;
  } else if ('User' in node) {
    return node.User.connectors;
  } else if ('Asset' in node) {
    return node.Asset.connectors;
  } else if ('Tag' in node) {
    return node.Tag.connectors;
  }
}

export const getPathAsString = (path: NodePath): string => {
  return path.map((g) => nodeNameAsString(g)).join(' ⇨ ');
};

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
      if (terms.every((t) => targetString.includes(t))) {
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
      terms.every((t) => targetString.includes(t));
    });
  }

  return result;
};
