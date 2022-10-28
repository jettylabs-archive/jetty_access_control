<template>
  <JettyTable
    title="All Tagged Assets"
    :rows-per-page="20"
    :row-transformer="rowTransformer"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="
      '/api/tag/' + encodeURIComponent(props.node.name) + '/all_assets'
    "
    v-slot="{ props: { row } }: { props: { row: AssetWithPaths } }"
    :tip="`Assets with the ${props.node.name} tag, either applied directly or through inheritance`"
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
import { AssetWithPaths } from '../models';
import { nodeNameAsString, getPathAsString } from 'src/util';
import NodePath from '../NodePath.vue';
import { mapNodeSummaryforSearch } from 'src/util/search';

const props = defineProps(['node']);

const columns = [
  {
    name: 'name',
    label: 'Asset Name',
    field: 'name',
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
  filename: props.node.name + '_all_assets.csv',
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
