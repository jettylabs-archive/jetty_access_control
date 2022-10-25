<template>
  <router-view />
</template>

<script lang="ts">
import { defineComponent } from 'vue';

export default defineComponent({
  name: 'App',
});
</script>

<script lang="ts" setup>
import { useJettyStore } from 'stores/jetty';

const store = useJettyStore();
store.fetchNodes();
store.fetchLastFetch();
</script>
