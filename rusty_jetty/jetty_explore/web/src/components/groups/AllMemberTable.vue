<template>
  <JettyTable
    title="All Group Members"
    :rows-per-page="20"
    :filter-method="filterMethod"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="
      '/api/group/' + encodeURIComponent(props.node.name) + '/all_members'
    "
    v-slot="{ props: { row } }: { props: { row: UserWithPaths } }"
    :tip="`All the members of ${props.node.name}, including the members
    inherited from child groups, when applicable`"
  >
    <q-tr>
      <q-td key="name">
        <q-item class="q-px-none">
          <q-item-section>
            <router-link
              :to="'/user/' + encodeURIComponent(nodeNameAsString(row.node))"
              style="text-decoration: none; color: inherit"
            >
              <q-item-label> {{ nodeNameAsString(row.node) }}</q-item-label>
            </router-link>

            <q-item-label caption>
              <JettyBadge
                v-for="connector in row.node.User.connectors"
                :key="connector"
                :name="connector"
              />
            </q-item-label>
          </q-item-section>
        </q-item>
      </q-td>
      <q-td key="membership_paths" class="q-px-none">
        <NodePath :paths="row.paths" />
      </q-td>
    </q-tr>
  </JettyTable>
</template>

<script lang="ts" setup>
import JettyTable from '../JettyTable.vue';
import JettyBadge from '../JettyBadge.vue';
import { NodePath as NodePathType, UserSummary } from '../models';
import { getPathAsString, nodeNameAsString } from 'src/util';
import NodePath from '../NodePath.vue';
import Fuse from 'fuse.js';

const props = defineProps(['node']);

interface UserWithPaths {
  node: UserSummary;
  paths: NodePathType[];
}

const columns = [
  {
    name: 'name',
    label: 'User',
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
const filterMethod = (rows: UserWithPaths[], terms) => {
  const fuse = new Fuse(rows, {
    keys: ['node.User.name.User', 'node.User.connectors'],
  });

  return fuse.search(terms).map((r) => r.item);

  const needles = terms.toLocaleLowerCase().split(' ');
  return rows.filter((r) =>
    needles.every(
      (needle) =>
        r.node.User.name.User.toLocaleLowerCase().indexOf(needle) > -1 ||
        // because we don't care which platform it matches, just concatenate them
        // into a single string
        r.node.User.connectors.join(' ').toLocaleLowerCase().indexOf(needle) >
          -1
    )
  );
};

const csvConfig = {
  filename: props.node.name + '_all_members.csv',
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
