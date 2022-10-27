<template>
  <div class="header flex column">
    <div class="title-and-icon flex q-pl-md">
      <q-icon :name="nodeIconFromNode(node)" color="primary" size="5em" />
      <div class="q-pl-md flex column">
        <text class="name">{{ nodeNameAsString(node) }}</text>
        <div class="header-badges">
          <JettyBadge
            v-for="platform in nodeConnectors(node)"
            :key="platform"
            :name="platform"
            big
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import JettyBadge from 'src/components/JettyBadge.vue';
import { nodeConnectors, nodeIconFromNode, nodeNameAsString } from 'src/util';

const props = defineProps(['node']);
</script>

<style lang="scss">
.header {
  padding-top: 40px;
}
.name {
  font-size: 25pt;
  font-weight: 200;
}
.title-and-icon {
  align-items: center;
}
.content {
  padding-top: 50px;
}
</style>
