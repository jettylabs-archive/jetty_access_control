<template>
  <router-link
    :to="'/user/' + nodeId(props.user)"
    style="text-decoration: none; color: inherit"
  >
    <q-item class="q-px-none">
      <q-item-section>
        <q-item-label class="text-weight-medium">
          {{ nodeNameAsString(props.user) }}</q-item-label
        >

        <q-item-label caption>
          <JettyBadge
            v-for="connector in user.User.connectors"
            :key="connector"
            :name="connector"
          />
        </q-item-label>
      </q-item-section>
    </q-item>
  </router-link>
</template>

<script lang="ts" setup>
import { nodeId, nodeNameAsString } from 'src/util';
import { UserSummary } from '../models';
import JettyBadge from '../JettyBadge.vue';

const props = defineProps<{ user: UserSummary }>();
</script>
