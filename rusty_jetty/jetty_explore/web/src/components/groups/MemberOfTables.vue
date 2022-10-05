<template>
  <div>
    <DirectMemberOf :node="props.node" />
    <InheritedMemberOf :node="props.node" />
  </div>
</template>

<script setup>
import DirectMemberOf from "./DirectMemberOf.vue";
import InheritedMemberOf from "./InheritedMemberOf.vue";

const props = defineProps(["node"]);
</script>
