<template>
  <div>
    <LineageUpstream :node="props.node" />
    <LineageDownstream :node="props.node" />
  </div>
</template>

<script setup>
import LineageUpstream from "./LineageUpstream.vue";
import LineageDownstream from "./LineageDownstream.vue";

const props = defineProps(["node"]);
</script>
