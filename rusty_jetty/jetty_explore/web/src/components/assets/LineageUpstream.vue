<template>
  <JettyTable
    title="Upstream Lineage"
    :rows-per-page="10"
    :filter-method="filterMethod"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="
      '/api/asset/' +
      encodeURIComponent(nodeNameAsString(props.node)) +
      '/lineage_upstream'
    "
    v-slot="{ props: { row } }: { props: { row: AssetWithPaths } }"
    :tip="`Assets upstream from ${nodeNameAsString(
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
import { getPathAsString, nodeNameAsString } from 'src/util';
import NodePath from '../NodePath.vue';
import AssetHeadline from './AssetHeadline.vue';

const props = defineProps(['node']);

// Filters by name, privileges, or connector
const filterMethod = (rows, terms) => {
  const needles = terms.toLocaleLowerCase().split(' ');
  return rows.filter((r) =>
    needles.every(
      (needle) =>
        r.name.toLocaleLowerCase().indexOf(needle) > -1 ||
        r.connector.toLocaleLowerCase().indexOf(needle) > -1
    )
  );
};

const columns = [
  {
    name: 'name',
    label: 'Asset Name',
    field: 'name',
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
  filename: props.node.name + '_upstream_assets_by_lineage.csv',
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
