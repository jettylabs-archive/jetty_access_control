<template>
  <router-link
    :to="'/asset/' + encodeURIComponent(asset.Asset.name.Asset.uri)"
    style="text-decoration: none; color: inherit"
  >
    <q-item class="q-px-none">
      <q-item-section>
        <q-item-label> {{ nodeNameAsString(asset) }}</q-item-label>
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

const props = defineProps<{ asset: AssetSummary }>();
</script>
