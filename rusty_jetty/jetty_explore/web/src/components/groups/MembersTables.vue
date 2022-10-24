<template>
  <div>
    <UserMembers :node="props.node" />
    <GroupMembers :node="props.node" />
  </div>
</template>

<script lang="ts" setup>
import GroupMembers from './GroupMembers.vue';
import UserMembers from './UserMembers.vue';

const props = defineProps(['node']);
</script>
