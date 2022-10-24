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
    v-slot="slotProps"
    :tip="`All the members of ${props.node.name}, including the members
    inherited from child groups, when applicable`"
  >
    <q-tr>
      <q-td key="name">
        <q-item class="q-px-none">
          <q-item-section>
            <router-link
              :to="'/user/' + encodeURIComponent(slotProps.props.row.name)"
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
      <q-td key="membership_paths" class="q-px-none">
        <div>
          <ul class="q-my-none q-pl-sm" style="list-style-type: '❯ '">
            <li
              v-for="path in slotProps.props.row.membership_paths"
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
const filterMethod = (rows, terms) => {
  const needles = terms.toLocaleLowerCase().split(' ');
  return rows.filter((r) =>
    needles.every(
      (needle) =>
        r.name.toLocaleLowerCase().indexOf(needle) > -1 ||
        // because we don't care which platform it matches, just concatenate them
        // into a single string
        r.platforms.join(' ').toLocaleLowerCase().indexOf(needle) > -1
    )
  );
};

const csvConfig = {
  filename: props.node.name + '_all_members.csv',
  columnNames: ['User', 'Platforms', 'Membership Path'],
  // accepts filtered sorted rows and returns the proper mapping
  mappingFn: (filteredSortedRows) =>
    filteredSortedRows.flatMap((r) =>
      r.membership_paths.map((m) => [r.name, r.platforms.join(', '), m])
    ),
};
</script>
