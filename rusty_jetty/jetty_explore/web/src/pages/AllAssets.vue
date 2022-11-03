<template>
  <q-page class="flex column container-md">
    <JettyTable
      title="All Assets"
      :rows-per-page="30"
      :row-transformer="rowTransformer"
      :columns="columns"
      :csv-config="csvConfig"
      fetchPath="/api/assets"
      v-slot="{ props: { row } }: { props: { row: AssetSummary } }"
    >
      <q-tr>
        <q-td key="name">
          <AssetHeadline :asset="row" />
        </q-td>
      </q-tr>
    </JettyTable>
  </q-page>
</template>

<script setup lang="ts">
import AssetHeadline from 'src/components/assets/AssetHeadline.vue';
import JettyBadge from 'src/components/JettyBadge.vue';
import JettyTable from 'src/components/JettyTable.vue';
import { AssetSummary } from 'src/components/models';
import { assetShortName, nodeConnectors, nodeNameAsString } from 'src/util';
import { mapNodeSummaryforSearch } from 'src/util/search';

const columns = [
  {
    name: 'name',
    label: 'Asset Name',
    sortable: true,
    align: 'left',
    // this must be unique, so combining the friendly short name with the unique full name
    field: (row: AssetSummary) => assetShortName(row) + nodeNameAsString(row),
  },
];

const rowTransformer = (row: AssetSummary): string =>
  mapNodeSummaryforSearch(row);

const csvConfig = {
  filename: 'assets.csv',
  columnNames: ['Asset Name', 'Platform'],
  // accepts a row and returns the proper mapping
  mappingFn: (filteredSortedRows: AssetSummary[]): string[][] =>
    filteredSortedRows.map((r) => [
      nodeNameAsString(r),
      nodeConnectors(r).join(', '),
    ]),
};
</script>
