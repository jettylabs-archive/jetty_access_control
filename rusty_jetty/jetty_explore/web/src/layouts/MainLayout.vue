<template>
  <q-layout view="hHh lpr fff">
    <q-header elevated>
      <q-toolbar>
        <router-link to="/" style="text-decoration: none; color: inherit">
          <q-toolbar-title class="toolbar-title">
            Jetty Explore
          </q-toolbar-title>
        </router-link>

        <div class="search-container flex">
          <SearchInput class="expanding-search" />
        </div>
      </q-toolbar>
    </q-header>

    <q-page-container>
      <router-view />
    </q-page-container>
  </q-layout>
</template>

<script setup lang="ts">
import SearchInput from 'src/components/SearchInput.vue';
</script>

<style lang="scss">
.expanding-search:focus-within {
  flex: 1;
  transition: flex 0.02s;
}
.search-container {
  flex: 1;
  justify-content: end;
}
.toolbar-title {
  min-width: 150px;
  flex: 0;
}
</style>
