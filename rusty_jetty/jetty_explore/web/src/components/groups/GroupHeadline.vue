<template>
  <router-link
    :to="'/group/' + nodeId(props.group)"
    style="text-decoration: none; color: inherit"
  >
    <q-item class="q-px-none">
      <q-item-section>
        <q-item-label class="text-weight-medium">
          {{ nodeNameAsString(props.group) }}</q-item-label
        >
        <q-item-label caption>
          <JettyBadge
            v-for="platform in props.group.Group.connectors"
            :key="platform"
            :name="platform"
          />
        </q-item-label>
      </q-item-section>
    </q-item>
  </router-link>
</template>

<script lang="ts" setup>
import { nodeNameAsString, nodeId } from 'src/util';
import { GroupSummary } from '../models';
import JettyBadge from '../JettyBadge.vue';

const props = defineProps<{ group: GroupSummary }>();
</script>
