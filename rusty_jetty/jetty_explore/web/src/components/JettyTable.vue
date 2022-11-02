<template>
  <div class="q-pa-md">
    <q-table
      :title="props.title"
      :loading="loading"
      :rows="rows"
      :columns="props.columns"
      row-key="name"
      flat
      :rows-per-page-options="[props.rowsPerPage]"
      :pagination="pagination"
      wrap-cells
      :filter="tableFilter"
      :filter-method="filterMethod"
      ref="jettyTable"
      dense
    >
      <template v-slot:top>
        <div class="flex justify-between fit">
          <div class="q-table__title">
            {{ props.title }}
            <q-badge v-if="props.tip" outline class="q-px-none" align="top">
              <q-icon name="help_outline" color="secondary" size="1rem">
                <q-tooltip class="text-body2 tooltip-wrap" max-width="300px">
                  {{ props.tip.replace('/', '/&shy;') }}
                </q-tooltip>
              </q-icon>
            </q-badge>
          </div>
          <div class="flex">
            <q-input
              outlined
              dense
              v-model="tableFilter"
              :debounce="debounceTime"
            >
              <template v-slot:prepend>
                <q-icon name="o_filter_alt" />
              </template>
            </q-input>
            <q-btn
              color="primary"
              icon-right="download"
              no-caps
              @click="exportTable"
              outline
              class="q-ml-xs q-px-sm"
            />
          </div>
        </div>
      </template>

      <template v-slot:body="props">
        <slot :props="props"> </slot>
      </template>
    </q-table>
  </div>
</template>

<script lang="ts" setup>
import { ref } from 'vue';
import { downloadCSV, fetchJson } from 'src/util';
import { jettySearch } from 'src/util/search';

const props = defineProps([
  'title',
  'rowsPerPage',
  'rowTransformer',
  'columns',
  'csvConfig',
  'fetchPath',
  'tip',
]);

var rows = ref([]);
var loading = ref(true);

const pagination = ref({
  sortBy: 'name',
  descending: false,
  page: 1,
  rowsPerPage: props.rowsPerPage,
});

const tableFilter = ref(null);

const jettyTable = ref(null);
const exportTable = () => {
  downloadCSV(
    props.csvConfig.filename,
    props.csvConfig.columnNames,
    props.csvConfig.mappingFn(jettyTable.value.filteredSortedRows)
  );
};

// we'll use this to keep the search feeling responsive
const debounceTime = ref(10);

const filterMethod = (rows: any[], terms) => {
  if (terms == '') {
    return rows;
  } else {
    var startTime = performance.now();
    const results = jettySearch(rows, props.rowTransformer, terms);
    debounceTime.value = Math.ceil(
      Math.max(debounceTime.value * 0.75, performance.now() - startTime)
    );
    return results;
  }
};

fetchJson(props.fetchPath)
  .then((r) => (rows.value = r))
  .catch((error) => console.log('unable to fetch: ', error))
  .finally(() => (loading.value = false));
</script>

<style lang="scss">
.tooltip-wrap {
  overflow-wrap: break-word;
  hyphenate-character: '';
}
</style>
