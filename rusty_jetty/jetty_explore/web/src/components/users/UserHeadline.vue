<template>
  <router-link
    :to="'/user/' + userpath"
    style="text-decoration: none; color: inherit"
  >
    <q-item class="q-px-none">
      <q-item-section>
        <q-item-label> {{ nodeNameAsString(user) }}</q-item-label>

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
import { nodeNameAsString } from 'src/util';
import { UserSummary } from '../models';
import JettyBadge from '../JettyBadge.vue';

const props = defineProps<{ user: UserSummary }>();
const userpath = encodeURIComponent(nodeNameAsString(props.user));
</script>
