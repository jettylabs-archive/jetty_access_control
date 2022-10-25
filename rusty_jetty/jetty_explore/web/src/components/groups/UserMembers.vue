<template>
  <JettyTable
    title="Direct Members - Users"
    :rows-per-page="10"
    :filter-method="filterMethod"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="
      '/api/group/' +
      encodeURIComponent(props.node.name) +
      '/direct_members_users'
    "
    v-slot="{ props: { row } }: { props: { row: UserSummary } }"
    :tip="`All the users who are explicitly assigned as members of ${props.node.name}`"
  >
    <q-tr>
      <q-td key="name">
        <q-item class="q-px-none">
          <q-item-section>
            <router-link
              :to="'/user/' + encodeURIComponent(nodeNameAsString(row))"
              style="text-decoration: none; color: inherit"
            >
              <q-item-label> {{ nodeNameAsString(row) }}</q-item-label>
            </router-link>
            <q-item-label caption>
              <JettyBadge
                v-for="connector in row.User.connectors"
                :key="connector"
                :name="connector"
              />
            </q-item-label>
          </q-item-section>
        </q-item>
      </q-td>
    </q-tr>
  </JettyTable>
</template>

<script lang="ts" setup>
import JettyTable from '../JettyTable.vue';
import JettyBadge from '../JettyBadge.vue';
import { UserSummary } from '../models';
import { nodeNameAsString } from 'src/util';

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
