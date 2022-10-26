<template>
  <q-page class="flex column container-md">
    <JettyHeader :node="currentNode" />
    <div class="q-px-md row items-start">
      <q-card flat class="tags-card q-mx-none">
        <q-card-section class="q-pa-xs">
          <div class="text-subtitle text-center text-weight-light q-py-xs">
            Direct Tags
          </div>
          <div class="flex justify-center">
            <JettyBadge
              v-for="tag in allTags.direct"
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
            <JettyBadge
              v-for="tag in allTags.via_hierarchy"
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
            <JettyBadge
              v-for="tag in allTags.via_lineage"
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
          label="Direct Access"
          :to="'/asset/' + encodeURIComponent(props.node_id) + '/direct_access'"
        />
        <q-route-tab
          name="all_users"
          label="Any Access"
          :to="'/asset/' + encodeURIComponent(props.node_id) + '/any_access'"
        />
        <q-route-tab
          name="hierarchy"
          label="Hierarchy"
          :to="'/asset/' + encodeURIComponent(props.node_id) + '/hierarchy'"
        />
        <q-route-tab
          name="lineage"
          label="Lineage"
          :to="'/asset/' + encodeURIComponent(props.node_id) + '/lineage'"
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

<script setup lang="ts">
import { ref, computed } from 'vue';
import JettyHeader from 'src/components/JettyHeader.vue';
import { useJettyStore } from 'stores/jetty';
import { useRouter, useRoute } from 'vue-router';
import JettyBadge from 'src/components/JettyBadge.vue';
import { fetchJson, nodeNameAsString } from 'src/util';
import { TagSummary } from 'src/components/models';

const props = defineProps(['node_id']);
const router = useRouter();
const route = useRoute();

const store = useJettyStore();
const nodeList = computed(() => store.nodes);
const currentNode = computed(() => {
  let returnNode;
  if (nodeList.value != null) {
    returnNode = nodeList.value.find(
      (node) => node.name == props.node_id && node.type == 'asset'
    );
  }
  return returnNode;
});

if (!currentNode.value) {
  router.push('/notfound');
}

const tab = ref('users');

interface TagResponse {
  direct: TagSummary[];
  via_lineage: TagSummary[];
  via_hierarchy: TagSummary[];
}

const allTags = ref<TagResponse>({
  direct: [],
  via_lineage: [],
  via_hierarchy: [],
});

fetchJson('/api/asset/' + encodeURIComponent(props.node_id) + '/tags')
  .then((r: TagResponse) => {
    console.log(r);
    allTags.value = r;
  })
  .catch((error) => console.log('unable to fetch: ', error));
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
.asset-content {
  padding-top: 25px;
}
.tags-card {
  flex: 1;
}
</style>
