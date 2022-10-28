<template>
  <JettyTable
    title="Directly Tagged Assets"
    :rows-per-page="20"
    :row-transformer="rowTransformer"
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
        <q-item class="q-px-none">
          <q-item-section>
            <router-link
              :to="'/group/' + encodeURIComponent(nodeNameAsString(row))"
              style="text-decoration: none; color: inherit"
            >
              <q-item-label> {{ nodeNameAsString(row) }}</q-item-label>
            </router-link>
            <q-item-label caption>
              <JettyBadge
                v-for="connector in row.Asset.connectors"
                :key="connector"
                :name="connector"
              />
            </q-item-label>
          </q-item-section>
        </q-item>
      </q-td>
    </q-tr>
  </JettyTable>
</template>

<script lang="ts" setup>
import JettyTable from '../JettyTable.vue';
import JettyBadge from '../JettyBadge.vue';
import { AssetSummary } from '../models';
import { nodeNameAsString } from 'src/util';
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
];

const rowTransformer = (row: AssetSummary): string =>
  mapNodeSummaryforSearch(row);

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
