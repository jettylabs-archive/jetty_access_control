<template>
  <JettyTable
    title="User-Accessible Tags"
    :rows-per-page="20"
    :filter-method="filterMethod"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="'/api/user/' + props.node.name + '/tags'"
    v-slot="slotProps"
    :tip="`The tags that ${props.node.name} has access to, through any asset privilege`"
  >
    <q-tr>
      <q-td key="name">
        <q-item class="q-px-none">
          <q-item-section>
            <router-link
              :to="'/tag/' + slotProps.props.row.name"
              style="text-decoration: none; color: inherit"
            >
              <q-item-label> {{ slotProps.props.row.name }}</q-item-label>
            </router-link>
          </q-item-section>
        </q-item>
      </q-td>
      <q-td key="assets" style="padding-right: 0px">
        <q-list dense>
          <q-item
            v-for="asset in slotProps.props.row.assets"
            :key="asset"
            class="q-px-none"
          >
            <div class="q-pr-sm">
              {{ asset.name }}
            </div>
            <div>
              <JettyBadge
                v-for="platform in slotProps.props.row.platforms"
                :key="platform"
                :name="platform"
              />
            </div>
          </q-item>
        </q-list>
      </q-td>
    </q-tr>
  </JettyTable>
</template>

<script setup>
import JettyTable from "../JettyTable.vue";
import JettyBadge from "../JettyBadge.vue";

const props = defineProps(["node"]);

const columns = [
  {
    name: "name",
    label: "Tag Name",
    field: "name",
    sortable: true,
    align: "left",
  },
  {
    name: "assets",
    label: "Accessible Assets",
    field: "assets",
    sortable: false,
    align: "left",
  },
];

// Filters by name, asset name, or asset platform
const filterMethod = (rows, terms) => {
  const needles = terms.toLocaleLowerCase().split(" ");
  return rows.filter((r) =>
    needles.every(
      (needle) =>
        r.name.toLocaleLowerCase().indexOf(needle) > -1 ||
        r.assets
          .map((a) => a.name)
          .join(" ")
          .toLocaleLowerCase()
          .indexOf(needle) > -1 ||
        r.assets
          .map((a) => a.platform)
          .join(" ")
          .toLocaleLowerCase()
          .indexOf(needle) > -1
    )
  );
};

const csvConfig = {
  filename: props.node.name + "_tags.csv",
  columnNames: ["Tag Name", "Accessible Asset", "Asset Platform"],
  // accepts a row and returns the proper mapping
  mappingFn: (filteredSortedRows) =>
    filteredSortedRows.flatMap((r) =>
      r.assets.map((a) => [r.name, a.name, a.platform])
    ),
};
</script>
