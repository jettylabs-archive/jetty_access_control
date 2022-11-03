<template>
  <JettyTable
    title="User Access"
    :rows-per-page="20"
    :row-transformer="rowTransformer"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="'/api/tag/' + nodeId(props.node) + '/users'"
    v-slot="{ props: { row } }: { props: { row: UserSummary } }"
    :tip="`Users with access to any asset with a ${nodeNameAsString(
      props.node
    )} tag`"
  >
    <q-tr>
      <q-td key="name">
        <UserHeadline :user="row" />
      </q-td>
    </q-tr>
  </JettyTable>
</template>

<script lang="ts" setup>
import JettyTable from '../JettyTable.vue';
import JettyBadge from '../JettyBadge.vue';
import { UserSummary } from '../models';
import { nodeNameAsString, nodeId } from 'src/util';
import { mapNodeSummaryforSearch } from 'src/util/search';
import UserHeadline from '../users/UserHeadline.vue';

const props = defineProps(['node']);

const columns = [
  {
    name: 'name',
    label: 'User',
    field: (row: UserSummary) => nodeNameAsString(row),
    sortable: true,
    align: 'left',
  },
];

const rowTransformer = (row: UserSummary): string =>
  mapNodeSummaryforSearch(row);

const csvConfig = {
  filename: nodeNameAsString(props.node) + '_user_access.csv',
  columnNames: ['User', 'Platforms'],
  // accepts filtered sorted rows and returns the proper mapping
  mappingFn: (filteredSortedRows: UserSummary[]) =>
    filteredSortedRows.map((r) => [
      nodeNameAsString(r),
      r.User.connectors.join(', '),
    ]),
};
</script>
