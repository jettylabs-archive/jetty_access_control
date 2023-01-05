<template>
  <q-page class="flex column container-md">
    <JettyHeader :node="currentNode" :subtitle="nodeNameAsString(currentNode)">
      <template #title>
        <text class="name"
          >{{ assetShortName(currentNode) }}
          <span class="name asset-type">
            ({{ currentNode.Asset.asset_type }})</span
          >
        </text>
      </template>
    </JettyHeader>

    <div class="q-px-md row items-start">
      <q-card flat class="tags-card q-mx-none">
        <q-card-section class="q-pa-xs">
          <div class="text-subtitle text-center text-weight-light q-py-xs">
            Direct Tags
          </div>
          <div class="flex justify-center">
            <span v-if="allTags.tags.direct.length === 0">None</span>
            <JettyBadge
              v-for="tag in allTags.tags.direct"
              :key="nodeNameAsString(tag)"
              :name="nodeNameAsString(tag)"
            />
          </div>
        </q-card-section>
      </q-card>
      <q-separator vertical inset class="q-mx-sm q-my-lg" />
      <q-card flat class="tags-card q-mx-none">
        <q-card-section class="q-pa-xs">
          <div class="text-subtitle text-center text-weight-light q-py-xs">
            Inherited Tags - Hierarchy
          </div>
          <div class="flex justify-center">
            <span v-if="allTags.tags.via_hierarchy.length === 0">None</span>
            <JettyBadge
              v-for="tag in allTags.tags.via_hierarchy"
              :key="nodeNameAsString(tag)"
              :name="nodeNameAsString(tag)"
            />
          </div>
        </q-card-section>
      </q-card>
      <q-separator vertical inset class="q-mx-sm q-my-lg" />
      <q-card flat class="tags-card q-mx-none">
        <q-card-section class="q-pa-xs">
          <div class="text-subtitle text-center text-weight-light q-py-xs">
            Inherited Tags - Lineage
          </div>
          <div class="flex justify-center">
            <span v-if="allTags.tags.via_lineage.length === 0">None</span>
            <JettyBadge
              v-for="tag in allTags.tags.via_lineage"
              :key="nodeNameAsString(tag)"
              :name="nodeNameAsString(tag)"
            />
          </div>
        </q-card-section>
      </q-card>
    </div>

    <div class="asset-content">
      <q-tabs
        dense
        class="text-grey col"
        active-color="primary"
        indicator-color="primary"
        align="justify"
        narrow-indicator
      >
        <q-route-tab
          name="users"
          label="Direct Access (preview)"
          :to="'/asset/' + props.node_id + '/direct_access'"
        />
        <q-route-tab
          name="all_users"
          label="Any Access (preview)"
          :to="'/asset/' + props.node_id + '/any_access'"
        />
        <q-route-tab
          name="hierarchy"
          label="Hierarchy"
          :to="'/asset/' + props.node_id + '/hierarchy'"
        />
        <q-route-tab
          name="lineage"
          label="Lineage"
          :to="'/asset/' + props.node_id + '/lineage'"
        />
      </q-tabs>

      <q-separator />

      <q-tab-panels animated v-model="tab">
        <q-tab-panel name="users">
          <router-view v-slot="{ Component }" :node="currentNode">
            <keep-alive max="6">
              <component :is="Component" :key="route.fullPath" />
            </keep-alive>
          </router-view>
        </q-tab-panel>

        <q-tab-panel name="all_users">
          <router-view :node="currentNode" />
        </q-tab-panel>
        <q-tab-panel name="hierarchy_upstream">
          <router-view :node="currentNode" />
        </q-tab-panel>
        <q-tab-panel name="hierarchy_downstream">
          <router-view :node="currentNode" />
        </q-tab-panel>
        <q-tab-panel name="lineage_upstream">
          <router-view :node="currentNode" />
        </q-tab-panel>
        <q-tab-panel name="lineage_downstream">
          <router-view :node="currentNode" />
        </q-tab-panel>
      </q-tab-panels>
    </div>
  </q-page>
</template>

<script lang="ts">
export default defineComponent({
  async beforeRouteUpdate(to, from) {
    if (to.path.split('/')[2] !== from.path.split('/')[2]) {
      this.updateTags(to.params.node_id);
    }
  },
});
</script>

<script setup lang="ts">
import { ref, computed, reactive, defineComponent } from 'vue';
import JettyHeader from 'src/components/JettyHeader.vue';
import { useJettyStore } from 'stores/jetty';
import { useRouter, useRoute } from 'vue-router';
import JettyBadge from 'src/components/JettyBadge.vue';
import { fetchJson, nodeId, nodeNameAsString, assetShortName } from 'src/util';
import { AssetSummary, TagSummary } from 'src/components/models';

const props = defineProps(['node_id']);
const router = useRouter();
const route = useRoute();

const store = useJettyStore();
const nodeList = computed(() => store.nodes);
const currentNode = computed(
  () =>
    nodeList.value.find((node) => nodeId(node) == props.node_id) as AssetSummary
);

if (!currentNode.value) {
  router.push('/notfound');
}

const tab = ref('users');

interface TagResponse {
  direct: TagSummary[];
  via_lineage: TagSummary[];
  via_hierarchy: TagSummary[];
}

const allTags: { tags: TagResponse } = reactive({
  tags: { direct: [], via_hierarchy: [], via_lineage: [] },
});

function updateTags(node_id: string) {
  fetchJson('/api/asset/' + node_id + '/tags')
    .then((r: TagResponse) => {
      allTags.tags = r;
    })
    .catch((error) => console.log('unable to fetch: ', error));
}

updateTags(props.node_id);

defineExpose({ updateTags });
</script>

<style lang="scss">
.asset-type {
  font-weight: 200 !important;
}
.asset-content {
  padding-top: 25px;
}
.tags-card {
  flex: 1;
}
</style>
