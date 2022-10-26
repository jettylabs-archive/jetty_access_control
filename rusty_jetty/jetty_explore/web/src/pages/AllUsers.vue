<template>
  <q-page class="flex column container-md">
    <JettyTable
      title="All Users"
      :rows-per-page="30"
      :filter-method="filterMethod"
      :columns="columns"
      :csv-config="csvConfig"
      fetchPath="/api/users"
      v-slot="slotProps"
    >
      <q-tr>
        <q-td key="name">
          <router-link
            :to="'/user/' + encodeURIComponent(slotProps.props.row.name)"
            style="text-decoration: none; color: inherit"
          >
            <q-item class="q-px-none">
              <q-item-section>
                <q-item-label> {{ slotProps.props.row.name }}</q-item-label>
                <q-item-label caption>
                  <JettyBadge
                    v-for="platform in slotProps.props.row.platforms"
                    :key="platform"
                    :name="platform"
                  />
                </q-item-label>
              </q-item-section>
            </q-item>
          </router-link>
        </q-td>
      </q-tr>
    </JettyTable>
  </q-page>
</template>

<script setup lang="ts">
import JettyBadge from 'src/components/JettyBadge.vue';
import JettyTable from 'src/components/JettyTable.vue';

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
