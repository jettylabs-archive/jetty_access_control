<template>
  <q-select
    outlined
    dense
    hide-dropdown-icon
    :model-value="model"
    use-input
    hide-selected
    fill-input
    :input-debounce="debounceTime"
    :options="options"
    @filter="filterFn"
    @input-value="setModel"
    :option-label="optionLabel"
    ref="searchField"
    bg-color="white"
    :autofocus="props.autofocus"
    @update:model-value="navigate"
  >
    <template v-slot:no-option>
      <q-item>
        <q-item-section class="text-grey"> No results </q-item-section>
      </q-item>
    </template>
    <template v-slot:prepend>
      <q-icon name="search" />
    </template>
    <template v-slot:option="scope">
      <AutocompleteItem :scope="scope" />
    </template>
  </q-select>
</template>

<script lang="ts" setup>
import { ref, computed } from 'vue';
import AutocompleteItem from './AutocompleteItem.vue';
import { useJettyStore } from 'stores/jetty';
import { useRouter } from 'vue-router';
import { jettySearch, mapNodeSummaryforSearch } from 'src/util/search';
import { nodeId, nodeNameAsString, NodeSummary, nodeType } from 'src/util';

const props = defineProps({
  autofocus: { type: Boolean },
});

const router = useRouter();
const store = useJettyStore();

const nodeOptions = computed(() => store.nodes);

const model = ref(null);
const options = ref<NodeSummary[]>([]);

const searchField = ref(null);

// we'll use this to keep the search feeling responsive
const debounceTime = ref(10);

// get the option label
const optionLabel = (item: NodeSummary | string): string => {
  if (typeof item === 'string') {
    return '';
  } else {
    return nodeNameAsString(item);
  }
};

function filterFn(val, update) {
  update(
    () => {
      if (val == '') {
        options.value = [];
      } else {
        var startTime = performance.now();
        options.value = jettySearch(
          nodeOptions.value,
          (i) => mapNodeSummaryforSearch(i),
          val,
          {
            numResults: 15,
          }
        );
        debounceTime.value = Math.ceil(
          Math.max(debounceTime.value * 0.75, performance.now() - startTime)
        );
      }
    },
    (ref) => {
      if (val !== '' && ref.options.length > 0) {
        ref.setOptionIndex(-1); // reset optionIndex in case there is something selected
        ref.moveOptionSelection(1, true); // focus the first selectable option and do not update the input-value
      }
    }
  );
}

function setModel(val) {
  model.value = val;
}

function navigate(val: NodeSummary) {
  model.value = null;
  let new_path = '/' + nodeType(val) + '/' + nodeId(val);
  if (searchField.value) {
    searchField.value.blur();
  }
  router.push(new_path);
}
</script>
