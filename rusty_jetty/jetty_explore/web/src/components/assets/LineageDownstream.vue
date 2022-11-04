<template>
  <JettyTable
    title="Downstream Lineage"
    :rows-per-page="10"
    :row-transformer="rowTransformer"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="'/api/asset/' + nodeId(props.node) + '/lineage_downstream'"
    v-slot="{ props: { row } }: { props: { row: AssetWithPaths } }"
    :tip="`Assets downstream from ${nodeNameAsString(
      props.node
    )}, based on data lineage`"
  >
    <q-tr>
      <q-td key="name">
        <AssetHeadline :asset="row.node" />
      </q-td>
      <q-td key="paths" class="q-px-none">
        <NodePath :paths="row.paths" />
      </q-td>
    </q-tr>
  </JettyTable>
</template>

<script lang="ts" setup>
import JettyTable from '../JettyTable.vue';
import { AssetWithPaths } from 'src/components/models';
import {
  getPathAsString,
  nodeNameAsString,
  nodeId,
  assetShortName,
} from 'src/util';
import NodePath from '../NodePath.vue';
import AssetHeadline from '../assets/AssetHeadline.vue';
import { mapNodeSummaryforSearch } from 'src/util/search';

const props = defineProps(['node']);

// Filters by name or platform
const rowTransformer = (row: AssetWithPaths): string =>
  mapNodeSummaryforSearch(row.node);

const columns = [
  {
    name: 'name',
    label: 'Asset Name',
    // this must be unique, so combining the friendly short name with the unique full name
    field: (row: AssetWithPaths) =>
      assetShortName(row.node) + nodeNameAsString(row.node),
    sortable: true,
    align: 'left',
  },
  {
    name: 'paths',
    label: 'Paths',
    field: 'paths',
    sortable: false,
    align: 'left',
  },
];

const csvConfig = {
  filename: nodeNameAsString(props.node) + '_downstream_assets_by_lineage.csv',
  columnNames: ['Asset Name', 'Asset Platform', 'Path'],
  // accepts a row and returns the proper mapping
  mappingFn: (filteredSortedRows: AssetWithPaths[]) =>
    filteredSortedRows.flatMap((r) =>
      r.paths.map((p) => [
        nodeNameAsString(r.node),
        r.node.Asset.connectors,
        getPathAsString(p),
      ])
    ),
};
</script>
