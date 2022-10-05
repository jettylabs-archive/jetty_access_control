<template>
  <router-view />
</template>

<script>
import { defineComponent } from "vue";

export default defineComponent({
  name: "App",
});
</script>

<script setup>
import { useJettyStore } from "stores/jetty";

const store = useJettyStore();
store.fetchNodes();
store.fetchLastFetch();
</script>
