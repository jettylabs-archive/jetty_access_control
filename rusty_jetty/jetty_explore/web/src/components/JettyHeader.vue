<template>
  <div class="header flex column">
    <div class="title-and-icon flex q-pl-md">
      <q-icon :name="nodeIconFromNode(node)" color="primary" size="5em" />
      <div class="q-pl-md flex column">
        <slot name="title">
          <text class="name">{{ nodeNameAsString(node) }}</text>
        </slot>
        <slot name="subtitle">
          <text v-if="props.subtitle" class="text-h6 text-weight-thin">
            {{ props.subtitle }}</text
          >
        </slot>
        <div class="header-badges">
          <JettyBadge
            v-for="platform in nodeConnectors(node)"
            :key="platform"
            :name="platform"
            big
          />
        </div>
        <slot></slot>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import JettyBadge from 'src/components/JettyBadge.vue';
import { nodeConnectors, nodeIconFromNode, nodeNameAsString } from 'src/util';

const props = defineProps(['node', 'subtitle']);
</script>

<style lang="scss">
.header {
  padding-top: 40px;
}
.name {
  font-size: 25pt;
  font-weight: 300;
}
.title-and-icon {
  align-items: start;
}
.header-badges {
  margin-left: -4px;
}
.content {
  padding-top: 30px;
}
</style>
