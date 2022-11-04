<template>
  <JettyTable
    title="Users with Access (Including via Linage)"
    :rows-per-page="20"
    :row-transformer="rowTransformer"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="'/api/asset/' + nodeId(props.node) + '/all_users'"
    v-slot="{ props: { row } }: { props: { row: UserWithAssets } }"
    :tip="`Users with access to ${nodeNameAsString(
      props.node
    )} or assets derived from ${nodeNameAsString(props.node)}`"
  >
    <q-tr>
      <q-td key="name">
        <UserHeadline :user="row.node" />
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
                :to="'/asset/' + nodeId(asset)"
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
import { nodeNameAsString, nodeId } from 'src/util';
import UserHeadline from '../users/UserHeadline.vue';
import { mapNodeSummaryforSearch } from 'src/util/search';

interface UserWithAssets {
  node: UserSummary;
  associations: AssetSummary[];
}

const props = defineProps(['node']);

const columns = [
  {
    name: 'name',
    label: 'User',
    field: (row: UserWithAssets) => nodeNameAsString(row.node),
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

const rowTransformer = (row: UserWithAssets): string =>
  mapNodeSummaryforSearch(row.node);

const csvConfig = {
  filename: nodeNameAsString(props.node) + '_users_with_any_access.csv',
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
