<template>
  <JettyTable
    title="Directly Tagged Assets"
    :rows-per-page="20"
    :row-transformer="rowTransformer"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="'/api/tag/' + nodeId(props.node) + '/direct_assets'"
    v-slot="{ props: { row } }: { props: { row: AssetSummary } }"
    :tip="`Assets directly tagged with ${nodeNameAsString(props.node)}`"
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
import { AssetSummary } from '../models';
import { nodeNameAsString, nodeId, assetShortName } from 'src/util';
import AssetHeadline from '../assets/AssetHeadline.vue';
import { mapNodeSummaryforSearch } from 'src/util/search';

const props = defineProps(['node']);

const columns = [
  {
    name: 'name',
    label: 'Asset Name',
    // this must be unique, so combining the friendly short name with the unique full name
    field: (row: AssetSummary) => assetShortName(row) + nodeNameAsString(row),
    sortable: true,
    align: 'left',
  },
];

const rowTransformer = (row: AssetSummary): string =>
  mapNodeSummaryforSearch(row);

const csvConfig = {
  filename: nodeNameAsString(props.node) + '_direct_assets.csv',
  columnNames: ['Asset Name', 'Asset Platform'],
  // accepts filtered sorted rows and returns the proper mapping
  mappingFn: (filteredSortedRows: AssetSummary[]) =>
    filteredSortedRows.map((r) => [
      nodeNameAsString(r),
      r.Asset.connectors.join(', '),
    ]),
};
</script>
