<template>
  <span>
    <q-chip
      v-if="props.big"
      class="q-mr-xs"
      outline
      square
      :style="{ color: getBadgeColor(props.name) }"
    >
      {{ props.name }}
    </q-chip>
    <q-badge
      v-else
      class="q-mr-xs"
      outline
      :style="{ color: getBadgeColor(props.name) }"
    >
      {{ props.name }}
    </q-badge>
  </span>
</template>

<script lang="ts" setup>
import { getBadgeColor } from 'src/util';

const props = defineProps({ name: String, big: Boolean });
</script>
