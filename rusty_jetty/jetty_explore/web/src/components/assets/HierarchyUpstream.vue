<template>
  <JettyTable
    title="Upstream Hierarchy"
    :rows-per-page="10"
    :filter-method="filterMethod"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="
      '/api/asset/' +
      encodeURIComponent(props.node.name) +
      '/hierarchy_upstream'
    "
    v-slot="slotProps"
    :tip="`Assets upstream from ${props.node.name}, based on object hierarchy`"
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
              <JettyBadge :name="slotProps.props.row.connector" />
            </q-item-label>
          </q-item-section>
        </q-item>
      </q-td>
      <q-td key="paths" class="q-px-none">
        <div>
          <ul class="q-my-none q-pl-sm" style="list-style-type: '❯ '">
            <li
              v-for="path in slotProps.props.row.paths"
              :key="path"
              style="padding-top: 2px; padding-bottom: 2px"
            >
              {{ path }}
            </li>
          </ul>
        </div>
      </q-td>
    </q-tr>
  </JettyTable>
</template>

<script lang="ts" setup>
import JettyTable from '../JettyTable.vue';
import JettyBadge from '../JettyBadge.vue';

const props = defineProps(['node']);

// Filters by name, privileges, or connector
const filterMethod = (rows, terms) => {
  const needles = terms.toLocaleLowerCase().split(' ');
  return rows.filter((r) =>
    needles.every(
      (needle) =>
        r.name.toLocaleLowerCase().indexOf(needle) > -1 ||
        r.connector.toLocaleLowerCase().indexOf(needle) > -1
    )
  );
};

const columns = [
  {
    name: 'name',
    label: 'Asset Name',
    field: 'name',
    sortable: true,
    align: 'left',
  },
  {
    name: 'paths',
    label: 'Paths',
    field: 'paths',
    sortable: false,
    align: 'left',
  },
];

const csvConfig = {
  filename: props.node.name + '_upstream_assets_by_hierarchy.csv',
  columnNames: ['Asset Name', 'Asset Platform', 'Path'],
  // accepts a row and returns the proper mapping
  mappingFn: (filteredSortedRows) =>
    filteredSortedRows.flatMap((r) =>
      r.paths.map((p) => [r.name, r.connector, p])
    ),
};
</script>
