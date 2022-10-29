<template>
  <q-page class="flex column container-md">
    <JettyTable
      title="All Users"
      :rows-per-page="30"
      :filter-method="filterMethod"
      :columns="columns"
      :csv-config="csvConfig"
      fetchPath="/api/users"
      v-slot="{ props: { row } }: { props: { row: UserSummary } }"
    >
      <q-tr>
        <q-td key="name">
          <UserHeadline :user="row" />
        </q-td>
      </q-tr>
    </JettyTable>
  </q-page>
</template>

<script setup lang="ts">
import JettyBadge from 'src/components/JettyBadge.vue';
import JettyTable from 'src/components/JettyTable.vue';
import { UserSummary } from 'src/components/models';
import UserHeadline from 'src/components/users/UserHeadline.vue';

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

// Filters by name, privileges, or platform
const filterMethod = (rows, terms) => {
  const needles = terms.toLocaleLowerCase().split(' ');
  return rows.filter((r) =>
    needles.every(
      (needle) =>
        r.name.toLocaleLowerCase().indexOf(needle) > -1 ||
        r.platforms.join(' ').toLocaleLowerCase().indexOf(needle) > -1
    )
  );
};

const csvConfig = {
  filename: 'users.csv',
  columnNames: ['User', 'Platforms'],
  // accepts a row and returns the proper mapping
  mappingFn: (filteredSortedRows) =>
    filteredSortedRows.map((r) => [r.name, r.platforms.join(', ')]),
};
</script>
