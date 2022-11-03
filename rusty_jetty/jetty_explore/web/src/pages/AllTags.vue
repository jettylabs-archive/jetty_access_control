<template>
  <q-page class="flex column container-md">
    <JettyTable
      title="All Tags"
      :rows-per-page="30"
      :row-transformer="rowTransformer"
      :columns="columns"
      :csv-config="csvConfig"
      fetchPath="/api/tags"
      v-slot="{ props: { row } }: { props: { row: TagSummary } }"
    >
      <q-tr>
        <q-td key="name">
          <TagHeadline :tag="row" />
        </q-td>
      </q-tr>
    </JettyTable>
  </q-page>
</template>

<script setup lang="ts">
import JettyTable from 'src/components/JettyTable.vue';
import { TagSummary } from 'src/components/models';
import TagHeadline from 'src/components/tags/TagHeadline.vue';
import { nodeConnectors, nodeNameAsString } from 'src/util';
import { mapNodeSummaryforSearch } from 'src/util/search';

const columns = [
  {
    name: 'name',
    label: 'Tag Name',
    field: (row: TagSummary) => nodeNameAsString(row),
    sortable: true,
    align: 'left',
  },
];

const rowTransformer = (row: TagSummary): string =>
  mapNodeSummaryforSearch(row);

const csvConfig = {
  filename: 'tags.csv',
  columnNames: ['Tag Name', 'Platforms'],
  // accepts a row and returns the proper mapping
  mappingFn: (filteredSortedRows: TagSummary[]): string[][] =>
    filteredSortedRows.map((r) => [
      nodeNameAsString(r),
      nodeConnectors(r).join(' '),
    ]),
};
</script>
