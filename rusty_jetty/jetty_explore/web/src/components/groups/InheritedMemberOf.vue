<template>
  <JettyTable
    title="Inherited Group Membership"
    :rows-per-page="10"
    :filter-method="filterMethod"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="
      '/api/group/' +
      encodeURIComponent(nodeNameAsString(props.node)) +
      '/inherited_groups'
    "
    v-slot="{ props: { row } }: { props: { row: GroupWithPaths } }"
    :tip="`The groups that ${nodeNameAsString(
      props.node
    )} is an inherited member of through child relationships`"
  >
    <q-tr>
      <q-td key="name">
        <GroupHeadline :group="row.node" />
      </q-td>
      <q-td key="membership_paths" class="q-px-none">
        <div>
          <NodePath :paths="row.paths" />
        </div>
      </q-td>
    </q-tr>
  </JettyTable>
</template>

<script lang="ts" setup>
import JettyTable from '../JettyTable.vue';
import NodePath from '../NodePath.vue';
import { GroupWithPaths } from '../models';
import { getPathAsString, nodeNameAsString } from 'src/util';
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
  {
    name: 'membership_paths',
    label: 'Membership Paths',
    field: 'membership_paths',
    sortable: false,
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
  filename: props.node.name + '_indirect_groups.csv',
  columnNames: ['Group Name', 'Platform', 'Membership Paths'],
  // accepts filtered sorted rows and returns the proper mapping
  mappingFn: (filteredSortedRows: GroupWithPaths[]) =>
    filteredSortedRows.flatMap((r) =>
      r.paths.map((m) => [
        nodeNameAsString(r.node),
        r.node.Group.connectors.join(', '),
        getPathAsString(m),
      ])
    ),
};
</script>
