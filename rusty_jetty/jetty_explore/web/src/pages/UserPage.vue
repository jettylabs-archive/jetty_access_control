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
      >
        <q-route-tab
          name="assets"
          label="Assets (preview)"
          :to="'/user/' + props.user_id + '/assets'"
        />
        <q-route-tab
          name="groups"
          label="Groups"
          :to="'/user/' + props.user_id + '/groups'"
        />
        <q-route-tab
          name="tags"
          label="Tags"
          :to="'/user/' + props.user_id + '/tags'"
        />
      </q-tabs>

      <q-separator />

      <q-tab-panels animated v-model="tab">
        <q-tab-panel name="assets">
          <router-view v-slot="{ Component }" :node="currentNode">
            <keep-alive max="3">
              <component :is="Component" :key="route.fullPath" />
            </keep-alive>
          </router-view>
        </q-tab-panel>

        <q-tab-panel name="groups">
          <router-view :node="currentNode" />
        </q-tab-panel>

        <q-tab-panel name="tags">
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
import { nodeId } from 'src/util';

const props = defineProps(['user_id']);
const route = useRoute();
const router = useRouter();

const store = useJettyStore();
const nodeList = computed(() => store.nodes);
const currentNode = computed(() => {
  let returnNode = {
    type: '',
    name: '',
    platforms: [],
  };
  if (nodeList.value != null) {
    returnNode = nodeList.value.find((node) => nodeId(node) == props.user_id);
  }
  return returnNode;
});

if (!currentNode.value) {
  router.push('/notfound');
}

const tab = ref('assets');
</script>
