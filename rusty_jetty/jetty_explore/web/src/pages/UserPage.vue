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
          label="Assets"
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
              <component :is="Component" />
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

<script setup>
import { ref, computed } from "vue";
import JettyHeader from "src/components/JettyHeader.vue";
import { useJettyStore } from "stores/jetty";
import { useRoute } from "vue-router";

const props = defineProps(["user_id"]);
const route = useRoute();

const store = useJettyStore();
const nodeList = computed(() => store.nodes);
const currentNode = computed(() => {
  let returnNode = {
    type: "",
    name: "",
    platforms: [],
  };
  if (nodeList.value != null) {
    returnNode = nodeList.value.find(
      (node) => node.name == props.user_id && node.type == "user"
    );
  }
  return returnNode;
});

if (!currentNode.value) {
  router.push("/notfound");
}

const tab = ref("assets");
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
.content {
  padding-top: 50px;
}
</style>
