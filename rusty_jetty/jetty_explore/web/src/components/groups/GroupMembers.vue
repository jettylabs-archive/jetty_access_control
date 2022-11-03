<template>
  <JettyTable
    title="Direct Members - Groups"
    :rows-per-page="10"
    :row-transformer="rowTransformer"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="'/api/group/' + nodeId(props.node) + '/direct_members_groups'"
    v-slot="{ props: { row } }: { props: { row: GroupSummary } }"
    :tip="`All the groups that are explicitly assigned as members of ${nodeNameAsString(
      props.node
    )}`"
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
import { GroupSummary } from '../models';
import { nodeNameAsString, nodeId } from 'src/util';
import GroupHeadline from './GroupHeadline.vue';
import { mapNodeSummaryforSearch } from 'src/util/search';

const props = defineProps(['node']);

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
  filename: nodeNameAsString(props.node) + '_direct_members_groups.csv',
  columnNames: ['Group Name', 'Platforms'],
  // accepts filtered sorted rows and returns the proper mapping
  mappingFn: (filteredSortedRows: GroupSummary[]) =>
    filteredSortedRows.map((r) => [
      nodeNameAsString(r),
      r.Group.connectors.join(', '),
    ]),
};
</script>
