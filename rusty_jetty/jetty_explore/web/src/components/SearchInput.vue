<template>
  <q-select
    outlined
    dense
    hide-dropdown-icon
    :model-value="model"
    use-input
    hide-selected
    fill-input
    input-debounce="0"
    :options="limitedOptions"
    @filter="filterFn"
    @input-value="setModel"
    option-label="name"
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

const props = defineProps({
  autofocus: { type: Boolean },
});

const router = useRouter();
const store = useJettyStore();

const nodeOptions = computed(() => store.nodes);

const model = ref(null);
const options = ref([]);

const searchField = ref(null);

const limitedOptions = computed(() => {
  return options.value.slice(0, 5);
});

function filterFn(val, update, abort) {
  update(
    () => {
      const needle = val.toLocaleLowerCase();
      options.value = nodeOptions.value.filter(
        (v) => v.name.toLocaleLowerCase().indexOf(needle) > -1
      );
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

function navigate(val) {
  model.value = null;
  let new_path = '/' + val.type + '/' + encodeURIComponent(val.name);
  if (searchField.value) {
    searchField.value.blur();
  }
  router.push(new_path);
}
</script>
