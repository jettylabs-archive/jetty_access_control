<template>
  <router-link
    :to="'/asset/' + nodeId(props.asset)"
    style="text-decoration: none; color: inherit"
  >
    <q-item class="q-px-none">
      <q-item-section>
        <q-item-label>
          <span class="q-pr-xs text-weight-medium">{{ assetShortname }} </span>
          <span class="text-caption"
            >({{ asset.Asset.asset_type }})</span
          ></q-item-label
        >
        <q-item-label caption> {{ nodeNameAsString(asset) }}</q-item-label>
        <q-item-label caption>
          <JettyBadge
            v-for="platform in asset.Asset.connectors"
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
import { AssetSummary } from '../models';
import JettyBadge from '../JettyBadge.vue';
import { computed } from 'vue';
import { nodeId } from 'src/util';

const props = defineProps<{ asset: AssetSummary }>();
const assetShortname = computed(() =>
  nodeNameAsString(props.asset).split('::').pop().split('/').pop()
);
</script>
