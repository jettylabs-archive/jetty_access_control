<template>
  <JettyTable
    title="User-Accessible Assets"
    :rows-per-page="20"
    :row-transformer="rowTransformer"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="'/api/user/' + encodeURIComponent(props.node.name) + '/assets'"
    v-slot="{
      props: { row },
    }: {
      props: { row: AssetWithEffectivePermissions },
    }"
    :tip="`All the assets ${props.node.name} has access too, including the privilege levels and
    sources of those privileges`"
  >
    <q-tr>
      <q-td key="name">
        <q-item class="q-px-none">
          <q-item-section>
            <router-link
              :to="'/asset/' + encodeURIComponent(nodeNameAsString(row.node))"
              style="text-decoration: none; color: inherit"
            >
              <q-item-label> {{ nodeNameAsString(row.node) }}</q-item-label>
            </router-link>

            <q-item-label caption>
              <JettyBadge
                v-for="connector in row.node.Asset.connectors"
                :key="connector"
                :name="connector"
              />
            </q-item-label>
          </q-item-section>
        </q-item>
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
import JettyBadge from '../JettyBadge.vue';
import { AssetSummary, EffectivePermission } from '../models';
import { nodeNameAsString } from 'src/util';
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
    field: 'name',
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
  filename: props.node.name + '_assets.csv',
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
