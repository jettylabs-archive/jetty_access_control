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

type GroupPath = GroupSummary[];

interface GroupSummary {
  Group: {
    name: {
      Group: {
        name: string,
        origin: string,
      }
    }
    connectors: string[]
  }
};

const props = defineProps<{ paths: GroupPath[] }>();

const getPathString = (path: GroupPath) => { console.log(path); return path.map(g => g.Group.name.Group.origin + '::' + g.Group.name.Group.name).join(' ⇨ ') };
</script>
