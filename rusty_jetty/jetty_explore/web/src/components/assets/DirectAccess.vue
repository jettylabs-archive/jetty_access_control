<template>
  <JettyTable
    title="Users with Direct Access"
    :rows-per-page="20"
    :filter-method="filterMethod"
    :columns="columns"
    :csv-config="csvConfig"
    :fetchPath="'/api/asset/' + encodeURIComponent(props.node.name) + '/users'"
    v-slot="slotProps"
    :tip="`Users with access to ${props.node.name}`"
  >
    <q-tr>
      <q-td key="name">
        <q-item class="q-px-none">
          <q-item-section>
            <router-link
              :to="'/user/' + encodeURIComponent(slotProps.props.row.name)"
              style="text-decoration: none; color: inherit"
            >
              <q-item-label> {{ slotProps.props.row.name }}</q-item-label>
            </router-link>

            <q-item-label caption>
              <JettyBadge
                v-for="platform in slotProps.props.row.platforms"
                :key="platform"
                :name="platform"
              />
            </q-item-label>
          </q-item-section>
        </q-item>
      </q-td>
      <q-td key="privileges" style="padding-right: 0px">
        <q-list separator>
          <q-item
            v-for="privilege in slotProps.props.row.privileges"
            :key="privilege"
            class="q-px-none"
          >
            <div class="q-pr-lg flex flex-center">
              {{ privilege.name }}
            </div>
            <div>
              <ul class="q-my-none" style="list-style-type: '❯ '">
                <li
                  v-for="explanation in privilege.explanations"
                  :key="explanation"
                  style="padding-top: 2px; padding-bottom: 2px"
                >
                  {{ explanation }}
                </li>
              </ul>
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

// Filters by name, privileges, or platform
const filterMethod = (rows, terms) => {
  const needles = terms.toLocaleLowerCase().split(" ");
  return rows.filter((r) =>
    needles.every(
      (needle) =>
        r.name.toLocaleLowerCase().indexOf(needle) > -1 ||
        r.platforms.join(" ").toLocaleLowerCase().indexOf(needle) > -1 ||
        r.privileges
          .map((a) => a.name)
          .join(" ")
          .toLocaleLowerCase()
          .indexOf(needle) > -1
    )
  );
};

const columns = [
  {
    name: "name",
    label: "User",
    field: "name",
    sortable: true,
    align: "left",
  },
  {
    name: "privileges",
    label: "Privilege and Explanation",
    field: "privileges",
    sortable: false,
    align: "left",
  },
];

const csvConfig = {
  filename: props.node.name + "_direct_access.csv",
  columnNames: ["Asset Name", "Privilege", "Explanation"],
  // accepts a row and returns the proper mapping
  mappingFn: (filteredSortedRows) =>
    filteredSortedRows.flatMap((r) =>
      r.privileges.flatMap((p) =>
        p.explanations.map((e) => [r.name, p.name, e])
      )
    ),
};
</script>
