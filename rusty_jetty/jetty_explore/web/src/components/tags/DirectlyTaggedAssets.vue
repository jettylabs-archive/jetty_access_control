<template>
  <JettyTable
    title="Directly Tagged Assets"
    :rows-per-page="20"
    :filter-method="filterMethod"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="
      '/api/tag/' + encodeURIComponent(props.node.name) + '/direct_assets'
    "
    v-slot="{ props: { row } }: { props: { row: AssetSummary } }"
    :tip="`Assets directly tagged with ${props.node.name}`"
  >
    <q-tr>
      <q-td key="name">
        <AssetHeadline :asset="row" />
      </q-td>
    </q-tr>
  </JettyTable>
</template>

<script lang="ts" setup>
import JettyTable from '../JettyTable.vue';
import JettyBadge from '../JettyBadge.vue';
import { AssetSummary } from '../models';
import { nodeNameAsString } from 'src/util';
import AssetHeadline from '../assets/AssetHeadline.vue';

const props = defineProps(['node']);

const columns = [
  {
    name: 'name',
    label: 'Asset Name',
    field: 'name',
    sortable: true,
    align: 'left',
  },
];

// Filters by name or platform
const filterMethod = (rows, terms) => {
  const needles = terms.toLocaleLowerCase().split(' ');
  return rows.filter((r) =>
    needles.every(
      (needle) =>
        r.name.toLocaleLowerCase().indexOf(needle) > -1 ||
        r.platform.toLocaleLowerCase().indexOf(needle) > -1
    )
  );
};

const csvConfig = {
  filename: props.node.name + '_direct_assets.csv',
  columnNames: ['Asset Name', 'Asset Platform'],
  // accepts filtered sorted rows and returns the proper mapping
  mappingFn: (filteredSortedRows: AssetSummary[]) =>
    filteredSortedRows.map((r) => [
      nodeNameAsString(r),
      r.Asset.connectors.join(', '),
    ]),
};
</script>
