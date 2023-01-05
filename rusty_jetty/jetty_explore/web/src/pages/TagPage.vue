<template>
  <q-page class="flex column container-md">
    <JettyHeader :node="currentNode">
      <template #subtitle>
        <text class="text-h6 text-weight-thin">
          {{ currentNode.Tag.description }}</text
        >
        <div class="flex row">
          <q-item class="q-pl-none"
            >{{
              currentNode.Tag.pass_through_lineage ? 'Passed' : 'Not passed'
            }}
            via lineage
          </q-item>
          <q-separator vertical inset class="q-mb-md" />
          <q-item
            >{{
              currentNode.Tag.pass_through_hierarchy ? 'Passed' : 'Not passed'
            }}
            via hierarchy
          </q-item>
        </div>
      </template>
    </JettyHeader>
    <div class="content">
      <q-tabs
        dense
        class="text-grey"
        active-color="primary"
        indicator-color="primary"
        align="justify"
        narrow-indicator
      >
        <q-route-tab
          name="all_assets"
          label="All Assets"
          :to="'/tag/' + props.node_id + '/all_assets'"
        />
        <q-route-tab
          name="direct_assets"
          label="Directly Tagged"
          :to="'/tag/' + props.node_id + '/direct_assets'"
        />
        <q-route-tab
          name="users"
          label="User Access (preview)"
          :to="'/tag/' + props.node_id + '/users'"
        />
      </q-tabs>

      <q-separator />

      <q-tab-panels animated v-model="tab">
        <q-tab-panel name="all_assets">
          <router-view v-slot="{ Component }" :node="currentNode">
            <keep-alive max="3">
              <component :is="Component" :key="route.fullPath" />
            </keep-alive>
          </router-view>
        </q-tab-panel>

        <q-tab-panel name="direct_assets">
          <router-view :node="currentNode" />
        </q-tab-panel>

        <q-tab-panel name="users">
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
import { useRoute, useRouter } from 'vue-router';
import { nodeId, NodeSummary } from 'src/util';
import { TagSummary } from 'src/components/models';

const route = useRoute();
const router = useRouter();

const props = defineProps(['node_id']);

const store = useJettyStore();
const nodeList = computed<NodeSummary[]>(() => store.nodes);
const currentNode = computed<TagSummary>(() => {
  return nodeList.value.find(
    (node) => nodeId(node) == props.node_id
  ) as TagSummary;
});

if (!currentNode.value) {
  router.push('/notfound');
}

const tab = ref('all_assets');
</script>
