<template>
  <span>
    <div>
      <ul class="q-my-none q-pl-sm" style="list-style-type: '❯ '">
        <li
          v-for="(item, idx) in props.paths"
          :key="idx"
          style="padding-top: 2px; padding-bottom: 2px"
        >
          {{ getPathString(item) }}
        </li>
      </ul>
    </div>
  </span>
</template>

<script lang="ts" setup>
import { nodeNameAsString } from 'src/util';
import { NodePath } from './models';

const props = defineProps<{ paths: NodePath[] }>();

const getPathString = (path: NodePath) => {
  console.log(path);
  return path.map((g) => nodeNameAsString(g)).join(' ⇨ ');
};
</script>
