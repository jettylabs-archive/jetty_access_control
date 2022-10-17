<template>
  <JettyTable
    title="Direct Members - Users"
    :rows-per-page="10"
    :filter-method="filterMethod"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="'/api/group/' + props.node.name + '/direct_members_users'"
    v-slot="slotProps"
    :tip="`All the users who are explicitly assigned as members of ${props.node.name}`"
  >
    <q-tr>
      <q-td key="name">
        <q-item class="q-px-none">
          <q-item-section>
            <router-link
              :to="'/user/' + slotProps.props.row.name"
              style="text-decoration: none; color: inherit"
            >
              <q-item-label> {{ slotProps.props.row.name }}</q-item-label>
            </router-link>
            <q-item-label caption>
              <JettyBadge
                v-for="platform in slotProps.props.row.connectors"
                :key="platform"
                :name="platform"
              />
            </q-item-label>
          </q-item-section>
        </q-item>
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
    label: "User",
    field: "name",
    sortable: true,
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
        r.connectors.join(", ").toLocaleLowerCase().indexOf(needle) > -1
    )
  );
};

const csvConfig = {
  filename: props.node.name + "_direct_members_users.csv",
  columnNames: ["User", "Platforms"],
  // accepts filtered sorted rows and returns the proper mapping
  mappingFn: (filteredSortedRows) =>
    filteredSortedRows.map((r) => [r.name, r.connectors.join(", ")]),
};
</script>
