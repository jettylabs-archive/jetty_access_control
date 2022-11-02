<template>
  <q-page class="flex column container-md">
    <JettyHeader :node="currentNode" />
    <div class="content">
      <q-tabs
        dense
        class="text-grey"
        active-color="primary"
        indicator-color="primary"
        align="justify"
        narrow-indicator
        v-model="tab"
      >
        <q-route-tab
          name="direct_members"
          label="Direct Members"
          :to="'/group/' + props.node_id + '/direct_members'"
        />
        <q-route-tab
          name="all_members"
          label="All Members"
          :to="'/group/' + props.node_id + '/all_members'"
        />
        <q-route-tab
          name="member_of"
          label="Member Of"
          :to="'/group/' + props.node_id + '/member_of'"
        />
      </q-tabs>

      <q-separator />

      <q-tab-panels animated v-model="tab">
        <q-tab-panel name="direct_members">
          <router-view v-slot="{ Component }" :node="currentNode">
            <keep-alive max="3">
              <component :is="Component" :key="route.fullPath" />
            </keep-alive>
          </router-view>
        </q-tab-panel>

        <q-tab-panel name="member_of">
          <router-view :node="currentNode" />
        </q-tab-panel>

        <q-tab-panel name="all_members">
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
import { nodeId } from 'src/util';

const props = defineProps(['node_id']);
const router = useRouter();
const route = useRoute();

const store = useJettyStore();
const nodeList = computed(() => store.nodes);
const currentNode = computed(() => {
  let returnNode;
  if (nodeList.value != null) {
    returnNode = nodeList.value.find((node) => nodeId(node) == props.node_id);
  }
  return returnNode;
});

if (!currentNode.value) {
  router.push('/notfound');
}

const tab = ref('direct_members');
</script>
