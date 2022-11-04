<template>
  <q-page class="flex column container-md">
    <JettyTable
      title="All Groups"
      :rows-per-page="30"
      :row-transformer="rowTransformer"
      :columns="columns"
      :csv-config="csvConfig"
      fetchPath="/api/groups"
      v-slot="{ props: { row } }: { props: { row: GroupSummary } }"
    >
      <q-tr>
        <q-td key="name">
          <GroupHeadline :group="row" />
        </q-td>
      </q-tr>
    </JettyTable>
  </q-page>
</template>

<script setup lang="ts">
import GroupHeadline from 'src/components/groups/GroupHeadline.vue';
import JettyTable from 'src/components/JettyTable.vue';
import { GroupSummary } from 'src/components/models';
import { nodeConnectors, nodeNameAsString } from 'src/util';
import { mapNodeSummaryforSearch } from 'src/util/search';

const columns = [
  {
    name: 'name',
    label: 'Group Name',
    field: (row: GroupSummary) => nodeNameAsString(row),
    sortable: true,
    align: 'left',
  },
];

const rowTransformer = (row: GroupSummary): string =>
  mapNodeSummaryforSearch(row);

const csvConfig = {
  filename: 'groups.csv',
  columnNames: ['Group Name', 'Platforms'],
  // accepts a row and returns the proper mapping
  mappingFn: (filteredSortedRows: GroupSummary[]): string[][] =>
    filteredSortedRows.map((r) => [
      nodeNameAsString(r),
      nodeConnectors(r).join(', '),
    ]),
};
</script>
