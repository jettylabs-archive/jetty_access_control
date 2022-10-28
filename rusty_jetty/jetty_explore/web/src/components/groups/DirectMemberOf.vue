<template>
  <JettyTable
    title="Direct Group Membership"
    :rows-per-page="10"
    :filter-method="filterMethod"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="
      '/api/group/' +
      encodeURIComponent(nodeNameAsString(props.node)) +
      '/direct_groups'
    "
    v-slot="{ props: { row } }: { props: { row: GroupSummary } }"
    :tip="`The groups that ${nodeNameAsString(
      props.node
    )} is a direct member of`"
  >
    <q-tr>
      <q-td key="name">
        <GroupHeadline :group="row" />
      </q-td>
    </q-tr>
  </JettyTable>
</template>

<script lang="ts" setup>
import JettyTable from '../JettyTable.vue';
import JettyBadge from '../JettyBadge.vue';
import { GroupSummary } from '../models';
import { nodeNameAsString } from 'src/util';
import GroupHeadline from './GroupHeadline.vue';

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

// Filters by name or platform
const filterMethod = (rows, terms) => {
  const needles = terms.toLocaleLowerCase().split(' ');
  return rows.filter((r) =>
    needles.every(
      (needle) =>
        r.name.toLocaleLowerCase().indexOf(needle) > -1 ||
        r.connectors.join(' ').toLocaleLowerCase().indexOf(needle) > -1
    )
  );
};

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
