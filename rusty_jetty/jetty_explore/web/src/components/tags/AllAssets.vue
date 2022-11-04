<template>
  <JettyTable
    title="All Tagged Assets"
    :rows-per-page="20"
    :row-transformer="rowTransformer"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="'/api/tag/' + nodeId(props.node) + '/all_assets'"
    v-slot="{ props: { row } }: { props: { row: AssetWithPaths } }"
    :tip="`Assets with the ${nodeNameAsString(
      props.node
    )} tag, either applied directly or through inheritance`"
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
import { AssetWithPaths } from '../models';
import {
  nodeNameAsString,
  getPathAsString,
  nodeId,
  assetShortName,
} from 'src/util';
import NodePath from '../NodePath.vue';
import { mapNodeSummaryforSearch } from 'src/util/search';
import AssetHeadline from '../assets/AssetHeadline.vue';

const props = defineProps(['node']);

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
    name: 'tag_paths',
    label: 'Tag Paths',
    field: 'tag_paths',
    sortable: false,
    align: 'left',
  },
];

const rowTransformer = (row: AssetWithPaths): string =>
  mapNodeSummaryforSearch(row.node);

const csvConfig = {
  filename: nodeNameAsString(props.node) + '_all_assets.csv',
  columnNames: ['Asset Name', 'Asset Platform', 'Tag Path'],
  // accepts filtered sorted rows and returns the proper mapping
  mappingFn: (filteredSortedRows: AssetWithPaths[]) =>
    filteredSortedRows.flatMap((r) =>
      r.paths.map((p) => [
        nodeNameAsString(r.node),
        r.node.Asset.connectors.join(', '),
        getPathAsString(p),
      ])
    ),
};
</script>
