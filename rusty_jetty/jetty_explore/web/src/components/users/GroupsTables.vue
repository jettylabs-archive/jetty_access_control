<template>
  <div>
    <DirectGroups :node="props.node" />
    <InheritedGroups :node="props.node" />
  </div>
</template>

<script setup>
import DirectGroups from "./DirectGroups.vue";
import InheritedGroups from "./InheritedGroups.vue";

const props = defineProps(["node"]);
</script>
