<template>
  <JettyTable
    title="Downstream Lineage"
    :rows-per-page="10"
    :row-transformer="rowTransformer"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="
      '/api/asset/' +
      encodeURIComponent(props.node.name) +
      '/lineage_downstream'
    "
    v-slot="{ props: { row } }: { props: { row: AssetWithPaths } }"
    :tip="`Assets downstream from ${props.node.name}, based on data lineage`"
  >
    <q-tr>
      <q-td key="name">
        <q-item class="q-px-none">
          <q-item-section>
            <router-link
              :to="'/asset/' + encodeURIComponent(nodeNameAsString(row.node))"
              style="text-decoration: none; color: inherit"
            >
              <q-item-label> {{ nodeNameAsString(row.node) }}</q-item-label>
            </router-link>
            <q-item-label caption>
              <JettyBadge
                v-for="connector in row.node.Asset.connectors"
                :key="connector"
                :name="connector"
              />
            </q-item-label>
          </q-item-section>
        </q-item>
      </q-td>
      <q-td key="paths" class="q-px-none">
        <NodePath :paths="row.paths" />
      </q-td>
    </q-tr>
  </JettyTable>
</template>

<script lang="ts" setup>
import JettyTable from '../JettyTable.vue';
import JettyBadge from '../JettyBadge.vue';
import { AssetWithPaths } from 'src/components/models';
import { getPathAsString, nodeNameAsString } from 'src/util';
import NodePath from '../NodePath.vue';
import { mapNodeSummaryforSearch } from 'src/util/search';

const props = defineProps(['node']);

// Filters by name or platform
const rowTransformer = (row: AssetWithPaths): string =>
  mapNodeSummaryforSearch(row.node);

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
  filename: props.node.name + '_downstream_assets_by_lineage.csv',
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
