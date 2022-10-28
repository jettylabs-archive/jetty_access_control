<template>
  <router-link
    :to="'/tag/' + encodeURIComponent(nodeNameAsString(tag))"
    style="text-decoration: none; color: inherit"
  >
    <q-item class="q-px-none">
      <q-item-section>
        <q-item-label class="text-weight-bold">
          {{ nodeNameAsString(tag) }}</q-item-label
        >
        <q-item-label caption>
          <JettyBadge
            v-for="platform in tag.Tag.connectors"
            :key="platform"
            :name="platform"
          />
        </q-item-label>
      </q-item-section>
    </q-item>
  </router-link>
</template>

<script lang="ts" setup>
import { nodeNameAsString } from 'src/util';
import { TagSummary } from '../models';
import JettyBadge from '../JettyBadge.vue';

const props = defineProps<{ tag: TagSummary }>();
</script>
