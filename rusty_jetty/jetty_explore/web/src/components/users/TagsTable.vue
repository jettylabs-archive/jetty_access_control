<template>
  <JettyTable
    title="User-Accessible Tags"
    :rows-per-page="20"
    :filter-method="filterMethod"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="'/api/user/' + encodeURIComponent(props.node.name) + '/tags'"
    v-slot="{ props: { row } }: { props: { row: TagWithAssets } }"
    :tip="`The tags that ${props.node.name} has access to, through any asset privilege`"
  >
    <q-tr>
      <q-td key="name">
        <q-item class="q-px-none">
          <q-item-section>
            <router-link
              :to="'/tag/' + encodeURIComponent(nodeNameAsString(row.node))"
              style="text-decoration: none; color: inherit"
            >
              <q-item-label> {{ nodeNameAsString(row.node) }}</q-item-label>
            </router-link>
          </q-item-section>
        </q-item>
      </q-td>
      <q-td key="assets" style="padding-right: 0px">
        <q-list dense>
          <q-item
            v-for="asset in row.list"
            :key="asset.Asset.name.Asset.uri"
            class="q-px-none"
          >
            <div class="q-pr-sm">
              {{ nodeNameAsString(asset) }}
            </div>
            <div>
              <JettyBadge
                v-for="connector in asset.Asset.connectors"
                :key="connector"
                :name="connector"
              />
            </div>
          </q-item>
        </q-list>
      </q-td>
    </q-tr>
  </JettyTable>
</template>

<script lang="ts" setup>
import JettyTable from '../JettyTable.vue';
import JettyBadge from '../JettyBadge.vue';
import { AssetSummary, TagSummary } from '../models';
import { nodeNameAsString } from 'src/util';

interface TagWithAssets {
  node: TagSummary;
  list: AssetSummary[];
}

const props = defineProps(['node']);

const columns = [
  {
    name: 'name',
    label: 'Tag Name',
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

// Filters by name, asset name, or asset platform
const filterMethod = (rows, terms) => {
  const needles = terms.toLocaleLowerCase().split(' ');
  return rows.filter((r) =>
    needles.every(
      (needle) =>
        r.name.toLocaleLowerCase().indexOf(needle) > -1 ||
        r.assets
          .map((a) => a.name)
          .join(' ')
          .toLocaleLowerCase()
          .indexOf(needle) > -1 ||
        r.assets
          .map((a) => a.platform)
          .join(' ')
          .toLocaleLowerCase()
          .indexOf(needle) > -1
    )
  );
};

const csvConfig = {
  filename: props.node.name + '_tags.csv',
  columnNames: ['Tag Name', 'Accessible Asset', 'Asset Platform'],
  // accepts a row and returns the proper mapping
  mappingFn: (filteredSortedRows: TagWithAssets[]) =>
    filteredSortedRows.flatMap((r) =>
      r.list.map((a) => [
        nodeNameAsString(r.node),
        nodeNameAsString(a),
        a.Asset.connectors.join(', '),
      ])
    ),
};
</script>
