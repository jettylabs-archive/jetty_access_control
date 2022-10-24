<template>
  <div>
    <HierarchyUpstream :node="props.node" />
    <HierarchyDownstream :node="props.node" />
  </div>
</template>

<script lang="ts" setup>
import HierarchyUpstream from './HierarchyUpstream.vue';
import HierarchyDownstream from './HierarchyDownstream.vue';

const props = defineProps(['node']);
</script>
