<template>
  <JettyTable title="Direct Group Membership" :rows-per-page="10" :filter-method="filterMethod" :columns="columns"
    :csv-config="csvConfig" :fetchPath="
      '/api/group/' + encodeURIComponent(props.node.name) + '/direct_groups'
    " v-slot="{ props: { row } }: { props: { row: GroupSummary } }"
    :tip="`The groups that ${props.node.name} is a direct member of`">
    <q-tr>
      <q-td key="name">
        <q-item class="q-px-none">
          <q-item-section>
            <router-link :to="'/group/' + encodeURIComponent(groupNameAsString(row.Group.name))"
              style="text-decoration: none; color: inherit">
              <q-item-label> {{ groupNameAsString(row.Group.name) }}</q-item-label>
            </router-link>
            <q-item-label caption>
              <JettyBadge v-for="connector in row.Group.connectors" :key="connector" :name="connector" />
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
import { GroupSummary } from '../models';
import { groupNameAsString } from 'src/util'

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
  mappingFn: (filteredSortedRows) =>
    filteredSortedRows.map((r) => [r.name, r.connectors.join(', ')]),
};
</script>
