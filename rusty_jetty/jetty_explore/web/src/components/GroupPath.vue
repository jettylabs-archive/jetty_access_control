<template>
  <span>
    <div>
      <ul class="q-my-none q-pl-sm" style="list-style-type: '❯ '">
        <li v-for="(item, idx) in props.paths" :key="idx" style="padding-top: 2px; padding-bottom: 2px">
          {{ getPathString(item) }}
        </li>
      </ul>
    </div>
  </span>
</template>

<script lang="ts" setup>
import { groupNameAsString } from 'src/util';
import { GroupPath } from './models';


const props = defineProps<{ paths: GroupPath[] }>();

const getPathString = (path: GroupPath) => { console.log(path); return path.map(g => groupNameAsString(g.Group.name)).join(' ⇨ ') };
</script>
