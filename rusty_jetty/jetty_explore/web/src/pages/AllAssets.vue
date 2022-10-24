<template>
  <q-page class="flex column container-md">
    <JettyTable
      title="All Assets"
      :rows-per-page="30"
      :filter-method="filterMethod"
      :columns="columns"
      :csv-config="csvConfig"
      fetchPath="/api/assets"
      v-slot="slotProps"
    >
      <q-tr>
        <q-td key="name">
          <q-item class="q-px-none">
            <q-item-section>
              <router-link
                :to="'/asset/' + encodeURIComponent(slotProps.props.row.name)"
                style="text-decoration: none; color: inherit"
              >
                <q-item-label> {{ slotProps.props.row.name }}</q-item-label>
              </router-link>
              <q-item-label caption>
                <JettyBadge
                  v-for="platform in slotProps.props.row.platforms"
                  :key="platform"
                  :name="platform"
                />
              </q-item-label>
            </q-item-section>
          </q-item>
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
    label: 'Asset Name',
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
  filename: 'assets.csv',
  columnNames: ['Asset Name', 'Platforms'],
  // accepts a row and returns the proper mapping
  mappingFn: (filteredSortedRows) =>
    filteredSortedRows.map((r) => [r.name, r.platforms.join(', ')]),
};
</script>
