<template>
  <JettyTable
    title="Users with Direct Access"
    :rows-per-page="20"
    :row-transformer="rowTransformer"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="'/api/asset/' + nodeId(props.node) + '/users'"
    v-slot="{
      props: { row },
    }: {
      props: { row: UserWithEffectivePermissions },
    }"
    :tip="`Users with access to ${nodeNameAsString(props.node)}`"
  >
    <q-tr>
      <q-td key="name">
        <UserHeadline :user="row.node" />
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
import UserHeadline from '../users/UserHeadline.vue';
import { EffectivePermission, UserSummary } from '../models';
import { nodeId, nodeNameAsString } from 'src/util';
import { mapNodeSummaryforSearch } from 'src/util/search';

interface UserWithEffectivePermissions {
  node: UserSummary;
  privileges: EffectivePermission[];
}

const props = defineProps(['node']);

const rowTransformer = (row: UserWithEffectivePermissions): string =>
  [
    mapNodeSummaryforSearch(row.node),
    ...row.privileges.map((p) => p.privilege),
  ].join(' ');

const columns = [
  {
    name: 'name',
    label: 'User',
    field: (row: UserWithEffectivePermissions) => nodeNameAsString(row.node),
    sortable: true,
    align: 'left',
  },
  {
    name: 'privileges',
    label: 'Privilege and Explanation',
    field: 'privileges',
    sortable: false,
    align: 'left',
  },
];

const csvConfig = {
  filename: nodeNameAsString(props.node) + '_direct_access.csv',
  columnNames: ['Asset Name', 'Privilege', 'Explanation'],
  // accepts a row and returns the proper mapping
  mappingFn: (filteredSortedRows: UserWithEffectivePermissions[]) =>
    filteredSortedRows.flatMap((r) =>
      r.privileges.flatMap((p) =>
        p.reasons.map((e) => [nodeNameAsString(r.node), p.privilege, e])
      )
    ),
};
</script>
