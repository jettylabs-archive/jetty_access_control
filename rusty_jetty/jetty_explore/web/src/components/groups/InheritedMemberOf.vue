<template>
  <JettyTable
    title="Inherited Group Membership"
    :rows-per-page="10"
    :row-transformer="rowTransformer"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="
      '/api/group/' + encodeURIComponent(props.node.name) + '/inherited_groups'
    "
    v-slot="{ props: { row } }: { props: { row: GroupWithPaths } }"
    :tip="`The groups that ${props.node.name} is an inherited member of through child relationships`"
  >
    <q-tr>
      <q-td key="name">
        <q-item class="q-px-none">
          <q-item-section>
            <router-link
              :to="'/group/' + nodeNameAsString(row.node)"
              style="text-decoration: none; color: inherit"
            >
              <q-item-label> {{ nodeNameAsString(row.node) }}</q-item-label>
            </router-link>
            <q-item-label caption>
              <JettyBadge
                v-for="connector in row.node.Group.connectors"
                :key="connector"
                :name="connector"
              />
            </q-item-label>
          </q-item-section>
        </q-item>
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
import JettyBadge from '../JettyBadge.vue';
import NodePath from '../NodePath.vue';
import { GroupWithPaths } from '../models';
import { getPathAsString, nodeNameAsString } from 'src/util';
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
  {
    name: 'membership_paths',
    label: 'Membership Paths',
    field: 'membership_paths',
    sortable: false,
    align: 'left',
  },
];

const rowTransformer = (row: GroupWithPaths): string =>
  mapNodeSummaryforSearch(row.node);

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
