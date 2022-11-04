<template>
  <q-page class="flex column container-md">
    <JettyTable
      title="All Users"
      :rows-per-page="30"
      :row-transformer="rowTransformer"
      :columns="columns"
      :csv-config="csvConfig"
      fetchPath="/api/users"
      v-slot="{ props: { row } }: { props: { row: UserSummary } }"
    >
      <q-tr>
        <q-td key="name">
          <UserHeadline :user="row" />
        </q-td>
      </q-tr>
    </JettyTable>
  </q-page>
</template>

<script setup lang="ts">
import JettyBadge from 'src/components/JettyBadge.vue';
import JettyTable from 'src/components/JettyTable.vue';
import { UserSummary } from 'src/components/models';
import UserHeadline from 'src/components/users/UserHeadline.vue';
import { nodeConnectors, nodeNameAsString } from 'src/util';
import { mapNodeSummaryforSearch } from 'src/util/search';

const columns = [
  {
    name: 'name',
    label: 'User',
    sortable: true,
    align: 'left',
    field: (row: UserSummary) => nodeNameAsString(row),
  },
];

const rowTransformer = (row: UserSummary): string =>
  mapNodeSummaryforSearch(row);

const csvConfig = {
  filename: 'users.csv',
  columnNames: ['User', 'Platforms'],
  // accepts a row and returns the proper mapping
  mappingFn: (filteredSortedRows: UserSummary[]): string[][] =>
    filteredSortedRows.map((r) => [
      nodeNameAsString(r),
      nodeConnectors(r).join(', '),
    ]),
};
</script>
