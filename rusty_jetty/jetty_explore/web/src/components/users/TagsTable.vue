<template>
  <JettyTable
    title="User-Accessible Tags"
    :rows-per-page="20"
    :row-transformer="rowTransformer"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="'/api/user/' + nodeId(props.node) + '/tags'"
    v-slot="{ props: { row } }: { props: { row: TagWithAssets } }"
    :tip="`The tags that ${nodeNameAsString(
      props.node
    )} has access to, through any asset privilege`"
  >
    <q-tr>
      <q-td key="name">
        <TagHeadline :tag="row.node" />
      </q-td>
      <q-td key="assets" style="padding-right: 0px">
        <ul class="q-my-none">
          <li
            v-for="asset in row.associations"
            :key="nodeNameAsString(asset)"
            style="padding-top: 2px; padding-bottom: 2px"
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
          </li>
        </ul>
      </q-td>
    </q-tr>
  </JettyTable>
</template>

<script lang="ts" setup>
import JettyTable from '../JettyTable.vue';
import JettyBadge from '../JettyBadge.vue';
import { AssetSummary, TagSummary } from '../models';
import { nodeNameAsString, nodeId } from 'src/util';
import { mapNodeSummaryforSearch } from 'src/util/search';
import TagHeadline from '../tags/TagHeadline.vue';

interface TagWithAssets {
  node: TagSummary;
  associations: AssetSummary[];
}

const props = defineProps(['node']);

const columns = [
  {
    name: 'name',
    label: 'Tag Name',
    field: (row: TagWithAssets) => nodeNameAsString(row.node),
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

const rowTransformer = (row: TagWithAssets): string =>
  [
    mapNodeSummaryforSearch(row.node),
    ...row.associations.map((a) => mapNodeSummaryforSearch(a)),
  ].join(' ');

const csvConfig = {
  filename: nodeNameAsString(props.node) + '_tags.csv',
  columnNames: ['Tag Name', 'Accessible Asset', 'Asset Platform'],
  // accepts a row and returns the proper mapping
  mappingFn: (filteredSortedRows: TagWithAssets[]) =>
    filteredSortedRows.flatMap((r) =>
      r.associations.map((a) => [
        nodeNameAsString(r.node),
        nodeNameAsString(a),
        a.Asset.connectors.join(', '),
      ])
    ),
};
</script>
