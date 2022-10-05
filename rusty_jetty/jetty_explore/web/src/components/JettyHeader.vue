<template>
  <div class="header flex column">
    <div class="title-and-icon flex q-pl-md">
      <q-icon :name="getNodeIcon(props.node.type)" color="primary" size="5em" />
      <div class="q-pl-md flex column">
        <text class="name">{{ props.node.name }}</text>
        <div class="header-badges">
          <JettyBadge
            v-for="platform in props.node.platforms"
            :key="platform"
            :name="platform"
            big
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from "vue";
import { getNodeIcon } from "src/util";
import JettyBadge from "src/components/JettyBadge.vue";

const props = defineProps(["node"]);

const tab = ref("assets");
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
