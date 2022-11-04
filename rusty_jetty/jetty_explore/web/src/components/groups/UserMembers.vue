<template>
  <JettyTable
    title="Direct Members - Users"
    :rows-per-page="10"
    :row-transformer="rowTransformer"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="'/api/group/' + nodeId(props.node) + '/direct_members_users'"
    v-slot="{ props: { row } }: { props: { row: UserSummary } }"
    :tip="`All the users who are explicitly assigned as members of ${nodeNameAsString(
      props.node
    )}`"
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
import { UserSummary } from '../models';
import { nodeNameAsString, nodeId } from 'src/util';
import UserHeadline from '../users/UserHeadline.vue';
import { mapNodeSummaryforSearch } from 'src/util/search';

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
  filename: nodeNameAsString(props.node) + '_direct_members_users.csv',
  columnNames: ['User', 'Platforms'],
  // accepts filtered sorted rows and returns the proper mapping
  mappingFn: (filteredSortedRows: UserSummary[]) =>
    filteredSortedRows.map((r) => [
      nodeNameAsString(r),
      r.User.connectors.join(', '),
    ]),
};
</script>
