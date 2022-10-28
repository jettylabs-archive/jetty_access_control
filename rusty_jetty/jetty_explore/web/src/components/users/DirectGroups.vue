<template>
  <JettyTable
    title="Direct Group Membership"
    :rows-per-page="10"
    :row-transformer="rowTransformer"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="
      '/api/user/' + encodeURIComponent(props.node.name) + '/direct_groups'
    "
    v-slot="{ props: { row } }: { props: { row: GroupSummary } }"
    :tip="`The groups that ${props.node.name} is a direct member of`"
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
                v-for="connector in row.Group.connectors"
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
import { GroupSummary } from '../models';
import { nodeNameAsString } from 'src/util';
import { mapNodeSummaryforSearch } from 'src/util/search';

const props = defineProps(['node']);

const columns = [
  {
    name: 'name',
    label: 'Group Name',
    field: 'name',
    sortable: true,
    align: 'left',
  },
];

const rowTransformer = (row: GroupSummary): string =>
  mapNodeSummaryforSearch(row);

const csvConfig = {
  filename: props.node.name + '_direct_groups.csv',
  columnNames: ['Group Name', 'Platforms'],
  // accepts filtered sorted rows and returns the proper mapping
  mappingFn: (filteredSortedRows: GroupSummary[]) =>
    filteredSortedRows.map((r) => [
      nodeNameAsString(r),
      r.Group.connectors.join(', '),
    ]),
};
</script>
