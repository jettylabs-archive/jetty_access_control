<template>
  <JettyTable
    title="User-Accessible Assets"
    :rows-per-page="20"
    :row-transformer="rowTransformer"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="'/api/user/' + nodeId(props.node) + '/assets'"
    v-slot="{
      props: { row },
    }: {
      props: { row: AssetWithEffectivePermissions },
    }"
    :tip="`All the assets ${nodeNameAsString(
      props.node
    )} has access too, including the privilege levels and
    sources of those privileges`"
  >
    <q-tr>
      <q-td key="name">
        <AssetHeadline :asset="row.node" />
      </q-td>
      <q-td key="privileges" style="padding-right: 0px">
        <q-list separator>
          <q-item
            v-for="privilege in row.privileges"
            :key="privilege.privilege"
            class="q-px-none"
          >
            <div class="q-pr-lg flex flex-center">
              {{ privilege.privilege }}
            </div>
            <div>
              <ul class="q-my-none">
                <li
                  v-for="reason in privilege.reasons"
                  :key="reason"
                  style="padding-top: 2px; padding-bottom: 2px"
                >
                  {{ reason }}
                </li>
              </ul>
            </div>
          </q-item>
        </q-list>
      </q-td>
    </q-tr>
  </JettyTable>
</template>

<script lang="ts" setup>
import JettyTable from '../JettyTable.vue';
import AssetHeadline from '../assets/AssetHeadline.vue';
import { AssetSummary, EffectivePermission } from '../models';
import { nodeNameAsString, nodeId, assetShortName } from 'src/util';
import { mapNodeSummaryforSearch } from 'src/util/search';

interface AssetWithEffectivePermissions {
  node: AssetSummary;
  privileges: EffectivePermission[];
}

const props = defineProps(['node']);

const rowTransformer = (row: AssetWithEffectivePermissions): string =>
  [
    mapNodeSummaryforSearch(row.node),
    ...row.privileges.map((p) => p.privilege),
  ].join(' ');

const columns = [
  {
    name: 'name',
    label: 'Asset Name',
    // this must be unique, so combining the friendly short name with the unique full name
    field: (row: AssetWithEffectivePermissions) =>
      assetShortName(row.node) + nodeNameAsString(row.node),
    sortable: true,
    align: 'left',
  },
  {
    name: 'privileges',
    label: 'Privileges',
    field: 'privileges',
    sortable: false,
    align: 'left',
  },
];

const csvConfig = {
  filename: nodeNameAsString(props.node) + '_assets.csv',
  columnNames: ['Asset Name', 'Asset Platform', 'Privilege', 'Explanation'],
  // accepts a row and returns the proper mapping
  mappingFn: (filteredSortedRows: AssetWithEffectivePermissions[]) =>
    filteredSortedRows.flatMap((r) =>
      r.privileges.flatMap((p) =>
        p.reasons.map((e) => [
          nodeNameAsString(r.node),
          r.node.Asset.connectors.join(', '),
          p.privilege,
          e,
        ])
      )
    ),
};
</script>
