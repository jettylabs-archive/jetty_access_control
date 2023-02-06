import { exportFile } from 'quasar';
import {
  AssetSummary,
  GroupSummary,
  NodePath,
  TagSummary,
  UserSummary,
} from './components/models';

export type NodeSummary =
  | AssetSummary
  | GroupSummary
  | UserSummary
  | TagSummary;

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

export const nodeIconFromNode = (node: NodeSummary) => {
  let icon = 'person';

  if ('Group' in node) {
    icon = 'group';
  } else if ('User' in node) {
    icon = 'person';
  } else if ('Asset' in node) {
    icon = 'table_chart';
  } else if ('Tag' in node) {
    icon = 'sell';
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

export function nodeNameAsString(node: NodeSummary): string {
  if ('Group' in node) {
    return node.Group.name.Group.origin + '::' + node.Group.name.Group.name;
  } else if ('User' in node) {
    return node.User.name.User;
  } else if ('Asset' in node) {
    return (
      node.Asset.name.Asset.connector +
      '::' +
      node.Asset.name.Asset.path.join('/')
    );
  } else if ('Tag' in node) {
    return node.Tag.name.Tag;
  }
}

export function nodeConnectors(node: NodeSummary): string[] {
  if ('Group' in node) {
    return node.Group.connectors;
  } else if ('User' in node) {
    return node.User.connectors;
  } else if ('Asset' in node) {
    return node.Asset.connectors;
  } else if ('Tag' in node) {
    return node.Tag.connectors;
  } else {
    return [];
  }
}

export const getPathAsString = (path: NodePath): string => {
  const path_names = path.map((n) => {
    if ('Asset' in n) {
      return assetShortName(n);
    } else {
      return nodeNameAsString(n);
    }
  });
  return path_names.join(' ⇨ ');
};

export function nodeId(node: NodeSummary): string {
  if ('Group' in node) {
    return node.Group.id;
  } else if ('User' in node) {
    return node.User.id;
  } else if ('Asset' in node) {
    return node.Asset.id;
  } else if ('Tag' in node) {
    return node.Tag.id;
  }
}

export function nodeType(node: NodeSummary): string {
  if ('Group' in node) {
    return 'group';
  } else if ('User' in node) {
    return 'user';
  } else if ('Asset' in node) {
    return 'asset';
  } else if ('Tag' in node) {
    return 'tag';
  }
}

export function assetShortName(asset: AssetSummary): string {
  return nodeNameAsString(asset).split('::').pop().split('/').pop();
}
