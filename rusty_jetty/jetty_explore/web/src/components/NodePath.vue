<template>
  <span>
    <div>
      <ul class="q-my-none q-pl-sm">
        <li
          v-for="(item, idx) in props.paths"
          :key="idx"
          style="padding-top: 2px; padding-bottom: 2px"
        >
          {{ getPathAsString(item) }}
        </li>
      </ul>
    </div>
  </span>
</template>

<script lang="ts" setup>
import { nodeNameAsString, getPathAsString } from 'src/util';
import { NodePath } from './models';

const props = defineProps<{ paths: NodePath[] }>();
</script>
