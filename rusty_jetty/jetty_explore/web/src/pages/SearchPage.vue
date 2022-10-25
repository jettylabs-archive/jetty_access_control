<template>
  <q-layout>
    <q-page-container>
      <q-page class="flex flex-center column">
        <div class="search-page-content flex flex-center column">
          <h2 class="q-mb-md">Jetty Explore</h2>
          <q-btn-group flat class="q-py-lg">
            <q-btn
              color="primary"
              flat
              label="Users"
              icon="person"
              to="/users"
            />
            <q-btn
              color="primary"
              flat
              label="Groups"
              icon="group"
              to="/groups"
            />
            <q-btn
              color="primary"
              flat
              label="Assets"
              icon="table_chart"
              to="/assets"
            />
            <q-btn color="primary" flat label="Tags" icon="sell" to="/tags" />
          </q-btn-group>
          <div class="main-search">
            <SearchInput :autofocus="true" />
          </div>
          <div class="jetty-info">
            <p class="text-body2 text-weight-light">
              Last fetch: {{ last_fetch }}
            </p>
            <p class="text-body2 text-weight-light">
              Run <i>jetty fetch</i> to update
            </p>
          </div>
        </div>
        <div class="search-page-spacer"></div>
      </q-page>
    </q-page-container>
  </q-layout>
</template>

<script setup lang="ts">
import SearchInput from 'src/components/SearchInput.vue';
import { computed } from 'vue';
import { useJettyStore } from 'stores/jetty';

const store = useJettyStore();
const last_fetch = computed(() =>
  store.last_fetch.toLocaleString(undefined, { timeZoneName: 'short' })
);
</script>

<style lang="scss">
.main-search {
  width: 500px;
  padding-bottom: 25px;
}
.search-page-spacer {
  flex: 2;
}
.search-page-content {
  flex: 3;
}
.jetty-info {
  text-align: center;
}
</style>
