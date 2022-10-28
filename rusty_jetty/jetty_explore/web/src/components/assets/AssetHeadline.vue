<template>
  <router-link
    :to="'/asset/' + encodeURIComponent(asset.Asset.name.Asset.uri)"
    style="text-decoration: none; color: inherit"
  >
    <q-item class="q-px-none">
      <q-item-section>
        <q-item-label
          ><span class="q-pr-xs text-weight-bold">{{ assetShortname }} </span>
          <span v-if="assetTypename" class="text-caption">{{
            `(${assetTypename})`
          }}</span></q-item-label
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

const props = defineProps<{ asset: AssetSummary }>();
const assetShortname = computed(() =>
  decodeURIComponent(
    new URL(nodeNameAsString(props.asset)).pathname.split('/').pop()
  )
);
const assetTypename = computed(
  () => new URL(nodeNameAsString(props.asset)).searchParams.get('type') || ''
);
</script>
