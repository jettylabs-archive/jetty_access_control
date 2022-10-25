<template>
  <q-item v-bind="props.scope.itemProps">
    <q-item-section avatar>
      <q-icon :name="getNodeIcon(scope.opt.type)" color="primary" />
    </q-item-section>
    <q-item-section>
      <q-item-label class="text-subtitle1 text-weight-light">
        {{ scope.opt.name }}
      </q-item-label>
      <q-item-label caption>
        <JettyBadge
          v-for="platform in scope.opt.platforms"
          v-bind:key="platform"
          :name="platform"
        >
        </JettyBadge>
      </q-item-label>
    </q-item-section>
  </q-item>
</template>

<script lang="ts" setup>
import { getNodeIcon } from '../util';
import JettyBadge from './JettyBadge.vue';

const props = defineProps(['scope']);
</script>

<style lang="scss"></style>
