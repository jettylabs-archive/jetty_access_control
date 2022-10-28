<template>
  <JettyTable
    title="Direct Members - Users"
    :rows-per-page="10"
    :filter-method="filterMethod"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="
      '/api/group/' +
      encodeURIComponent(nodeNameAsString(props.node)) +
      '/direct_members_users'
    "
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
import JettyBadge from '../JettyBadge.vue';
import { UserSummary } from '../models';
import { nodeNameAsString } from 'src/util';
import UserHeadline from '../users/UserHeadline.vue';

const props = defineProps(['node']);

const columns = [
  {
    name: 'name',
    label: 'User',
    field: 'name',
    sortable: true,
    align: 'left',
  },
];

// Filters by name or platform
const filterMethod = (rows, terms) => {
  const needles = terms.toLocaleLowerCase().split(' ');
  return rows.filter((r) =>
    needles.every(
      (needle) =>
        r.name.toLocaleLowerCase().indexOf(needle) > -1 ||
        r.connectors.join(', ').toLocaleLowerCase().indexOf(needle) > -1
    )
  );
};

const csvConfig = {
  filename: props.node.name + '_direct_members_users.csv',
  columnNames: ['User', 'Platforms'],
  // accepts filtered sorted rows and returns the proper mapping
  mappingFn: (filteredSortedRows: UserSummary[]) =>
    filteredSortedRows.map((r) => [
      nodeNameAsString(r),
      r.User.connectors.join(', '),
    ]),
};
</script>
