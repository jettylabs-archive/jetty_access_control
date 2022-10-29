<template>
  <q-page class="flex column container-md">
    <JettyTable
      title="All Groups"
      :rows-per-page="30"
      :filter-method="filterMethod"
      :columns="columns"
      :csv-config="csvConfig"
      fetchPath="/api/groups"
      v-slot="slotProps"
    >
      <q-tr>
        <q-td key="name">
          <GroupHeadline :group="slotProps.props.row" />
        </q-td>
      </q-tr>
    </JettyTable>
  </q-page>
</template>

<script setup lang="ts">
import GroupHeadline from 'src/components/groups/GroupHeadline.vue';
import JettyTable from 'src/components/JettyTable.vue';

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
  filename: 'groups.csv',
  columnNames: ['Group Name', 'Platforms'],
  // accepts a row and returns the proper mapping
  mappingFn: (filteredSortedRows) =>
    filteredSortedRows.map((r) => [r.name, r.platforms.join(', ')]),
};
</script>
