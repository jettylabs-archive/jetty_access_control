import { defineStore } from 'pinia';
import { fetchJson } from 'src/util';

export const useJettyStore = defineStore('jetty', {
  state: () => ({
    nodes: [],
    last_fetch: null,
  }),

  getters: {
    getNodes(state) {
      return state.nodes;
    },
  },

  actions: {
    fetchNodes() {
      fetchJson('/api/nodes')
        .then((r) => {
          if (r) {
            this.nodes = r;
          }
        })
        .catch((error) => console.log('error fetching data:', error));
    },

    fetchLastFetch() {
      fetchJson('/api/last_fetch')
        .then((r) => {
          if (r) {
            this.last_fetch = new Date(r.last_fetch_timestamp * 1000);
          }
        })
        .catch((error) => console.log('error fetching data:', error));
    },
  },
});
