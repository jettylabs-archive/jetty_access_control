<template>
  <JettyTable
    title="Inherited Group Membership"
    :rows-per-page="10"
    :filter-method="filterMethod"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="'/api/group/' + encodeURIComponent(props.node.name) + '/inherited_groups'"
    v-slot="slotProps"
    :tip="`The groups that ${props.node.name} is an inherited member of through child relationships`"
  >
    <q-tr>
      <q-td key="name">
        <q-item class="q-px-none">
          <q-item-section>
            <router-link
              :to="'/group/' + encodeURIComponent(slotProps.props.row.name)"
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
      <q-td key="membership_paths" class="q-px-none">
        <div>
          <ul class="q-my-none q-pl-sm" style="list-style-type: '❯ '">
            <li
              v-for="path in slotProps.props.row.membership_paths"
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
    label: "Group Name",
    field: "name",
    sortable: true,
    align: "left",
  },
  {
    name: "membership_paths",
    label: "Membership Paths",
    field: "membership_paths",
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
        r.connectors.join(" ").toLocaleLowerCase().indexOf(needle) > -1
    )
  );
};

const csvConfig = {
  filename: props.node.name + "_indirect_groups.csv",
  columnNames: ["Group Name", "Platform", "Membership Paths"],
  // accepts filtered sorted rows and returns the proper mapping
  mappingFn: (filteredSortedRows) =>
    filteredSortedRows.flatMap((r) =>
      r.membership_paths.map((m) => [r.name, r.connectors.join(", "), m])
    ),
};
</script>
