<template>
  <JettyTable
    title="Users with Access (Including via Linage)"
    :rows-per-page="20"
    :filter-method="filterMethod"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="
      '/api/asset/' + encodeURIComponent(props.node.name) + '/all_users'
    "
    v-slot="{ props: { row } }: { props: { row: UserWithAssets } }"
    :tip="`Users with access to ${props.node.name} or assets derived from ${props.node.name}`"
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
      <q-td key="assets" class="q-px-none">
        <div>
          <ul class="q-my-none q-pl-sm">
            <li
              v-for="asset in row.associations"
              :key="nodeNameAsString(asset)"
              style="padding-top: 2px; padding-bottom: 2px"
            >
              <router-link
                :to="'/asset/' + encodeURIComponent(nodeNameAsString(asset))"
                style="text-decoration: none; color: inherit"
              >
                {{ nodeNameAsString(asset) }}
              </router-link>
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
import { AssetSummary, UserSummary } from '../models';
import { nodeNameAsString } from 'src/util';

interface UserWithAssets {
  node: UserSummary;
  associations: AssetSummary[];
}

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
    name: 'assets',
    label: 'Accessible Assets',
    field: 'assets',
    sortable: false,
    align: 'left',
  },
];

// Filters by name, privileges, or connector
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
  filename: props.node.name + '_users_with_any_access.csv',
  columnNames: ['User', 'Platforms', 'Accessible Asset'],
  // accepts a row and returns the proper mapping
  mappingFn: (filteredSortedRows: UserWithAssets[]) =>
    filteredSortedRows.flatMap((r) =>
      r.associations.map((a) => [
        nodeNameAsString(r.node),
        r.node.User.connectors.join(', '),
        nodeNameAsString(a),
      ])
    ),
};
</script>
