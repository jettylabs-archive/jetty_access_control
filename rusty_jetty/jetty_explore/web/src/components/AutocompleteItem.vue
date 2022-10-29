<template>
  <q-item v-bind="props.scope.itemProps">
    <q-item-section avatar>
      <q-icon :name="nodeIconFromNode(scope.opt)" color="primary" />
    </q-item-section>

    <q-item-section>
      <GroupHeadline v-if="'Group' in scope.opt" :group="scope.opt" />
      <TagHeadline v-else-if="'Tag' in scope.opt" :tag="scope.opt" />
      <UserHeadline v-else-if="'User' in scope.opt" :user="scope.opt" />
      <AssetHeadline v-else-if="'Asset' in scope.opt" :asset="scope.opt" />
    </q-item-section>
  </q-item>
</template>

<script lang="ts" setup>
import { nodeIconFromNode } from '../util';
import AssetHeadline from './assets/AssetHeadline.vue';
import GroupHeadline from './groups/GroupHeadline.vue';
import TagHeadline from './tags/TagHeadline.vue';
import UserHeadline from './users/UserHeadline.vue';

const props = defineProps(['scope']);
</script>

<style lang="scss"></style>
