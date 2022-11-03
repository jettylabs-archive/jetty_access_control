<template>
  <JettyTable
    title="All Group Members"
    :rows-per-page="20"
    :row-transformer="rowTransformer"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="'/api/group/' + nodeId(props.node) + '/all_members'"
    v-slot="{ props: { row } }: { props: { row: UserWithPaths } }"
    :tip="`All the members of ${nodeNameAsString(
      props.node
    )}, including the members
    inherited from child groups, when applicable`"
  >
    <q-tr>
      <q-td key="name">
        <UserHeadline :user="row.node" />
      </q-td>
      <q-td key="membership_paths" class="q-px-none">
        <NodePath :paths="row.paths" />
      </q-td>
    </q-tr>
  </JettyTable>
</template>

<script lang="ts" setup>
import JettyTable from '../JettyTable.vue';
import { NodePath as NodePathType, UserSummary } from '../models';
import { getPathAsString, nodeNameAsString, nodeId } from 'src/util';
import NodePath from '../NodePath.vue';
import UserHeadline from '../users/UserHeadline.vue';
import { mapNodeSummaryforSearch } from 'src/util/search';

const props = defineProps(['node']);

interface UserWithPaths {
  node: UserSummary;
  paths: NodePathType[];
}

const columns = [
  {
    name: 'name',
    label: 'User',
    field: (row: UserWithPaths) => nodeNameAsString(row.node),
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

const rowTransformer = (row: UserWithPaths): string =>
  mapNodeSummaryforSearch(row.node);

const csvConfig = {
  filename: nodeNameAsString(props.node) + '_all_members.csv',
  columnNames: ['User', 'Platforms', 'Membership Path'],
  // accepts filtered sorted rows and returns the proper mapping
  mappingFn: (filteredSortedRows: UserWithPaths[]) =>
    filteredSortedRows.flatMap((r) =>
      r.paths.map((m) => [
        nodeNameAsString(r.node),
        r.node.User.connectors.join(', '),
        getPathAsString(m),
      ])
    ),
};
</script>
