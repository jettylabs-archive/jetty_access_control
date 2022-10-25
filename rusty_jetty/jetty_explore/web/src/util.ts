import { exportFile } from 'quasar';

const getBadgeColor = (stringInput: string) => {
  if (stringInput.toLocaleLowerCase() == 'jetty') {
    return '#f47124';
  }

  // Credit to https://stackoverflow.com/a/66494926
  const stringUniqueHash = [...stringInput].reduce((acc, char) => {
    return char.charCodeAt(0) + ((acc << 5) - acc);
  }, 0);
  return `hsl(${stringUniqueHash % 360}, 95%, 35%)`;
};

const getNodeIcon = (stringInput: string) => {
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

function wrapCsvValue(val) {
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

function downloadCSV(filename, columns, rows) {
  // naive encoding to csv format
  const content = [columns.map((c) => wrapCsvValue(c))]
    .concat(rows.map((row) => row.map((val) => wrapCsvValue(val)).join(',')))
    .join('\r\n');

  const status = exportFile(filename, content, 'text/csv');

  if (status !== true) {
    console.log('Browser denied file download...');
  }
}

function fetchJson(path: string) {
  const requestOptions: RequestInit = {
    method: 'GET',
    redirect: 'follow',
  };

  return fetch(path, requestOptions)
    .then((response) => response.json())
    .catch((error) => console.log('error fetching data:', error));
}

export { getBadgeColor, downloadCSV, fetchJson, getNodeIcon };
