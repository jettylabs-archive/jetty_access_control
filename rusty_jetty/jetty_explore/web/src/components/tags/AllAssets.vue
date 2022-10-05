<template>
  <JettyTable
    title="All Tagged Assets"
    :rows-per-page="20"
    :filter-method="filterMethod"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="'/api/tag/' + props.node.name + '/all_assets'"
    v-slot="slotProps"
    :tip="`Assets with the ${props.node.name} tag, either applied directly or through inheritance`"
  >
    <q-tr>
      <q-td key="name">
        <q-item class="q-px-none">
          <q-item-section>
            <router-link
              :to="'/asset/' + slotProps.props.row.name"
              style="text-decoration: none; color: inherit"
            >
              <q-item-label> {{ slotProps.props.row.name }}</q-item-label>
            </router-link>
            <q-item-label caption>
              <JettyBadge :name="slotProps.props.row.platform" />
            </q-item-label>
          </q-item-section>
        </q-item>
      </q-td>
      <q-td key="tag_paths" class="q-px-none">
        <div>
          <ul class="q-my-none q-pl-sm" style="list-style-type: '❯ '">
            <li
              v-for="path in slotProps.props.row.tag_paths"
              :key="path"
              style="padding-top: 2px; padding-bottom: 2px"
            >
              {{ path }}
            </li>
          </ul>
        </div>
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
    label: "Asset Name",
    field: "name",
    sortable: true,
    align: "left",
  },
  {
    name: "tag_paths",
    label: "Tag Paths",
    field: "tag_paths",
    sortable: false,
    align: "left",
  },
];

// Filters by name or platform
const filterMethod = (rows, terms) => {
  const needles = terms.toLocaleLowerCase().split(" ");
  return rows.filter((r) =>
    needles.every(
      (needle) =>
        r.name.toLocaleLowerCase().indexOf(needle) > -1 ||
        r.platform.toLocaleLowerCase().indexOf(needle) > -1
    )
  );
};

const csvConfig = {
  filename: props.node.name + "_all_assets.csv",
  columnNames: ["Asset Name", "Asset Platform", "Tag Path"],
  // accepts filtered sorted rows and returns the proper mapping
  mappingFn: (filteredSortedRows) =>
    filteredSortedRows.flatMap((r) =>
      r.tag_paths.map((p) => [r.name, r.platform, p])
    ),
};
</script>
