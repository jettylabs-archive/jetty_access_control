import { RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  { path: '/search', component: () => import('pages/SearchPage.vue') },
  {
    path: '/',
    component: () => import('layouts/MainLayout.vue'),
    children: [
      { path: '', redirect: '/search' },
      {
        path: '/users',
        component: () => import('pages/AllUsers.vue'),
      },
      {
        path: '/groups',
        component: () => import('pages/AllGroups.vue'),
      },
      {
        path: '/tags',
        component: () => import('pages/AllTags.vue'),
      },
      {
        path: '/assets',
        component: () => import('pages/AllAssets.vue'),
      },
      {
        path: '/user/:user_id',
        component: () => import('pages/UserPage.vue'),
        props: true,
        children: [
          { path: '', redirect: { name: 'assets' } },
          {
            name: 'assets',
            path: '/user/:user_id/assets',
            component: () => import('components/users/AssetsTable.vue'),
          },
          {
            path: '/user/:user_id/tags',
            component: () => import('components/users/TagsTable.vue'),
          },
          {
            path: '/user/:user_id/groups',
            component: () => import('components/users/GroupsTables.vue'),
          },
        ],
      },
      {
        path: '/group/:node_id',
        component: () => import('pages/GroupPage.vue'),
        props: true,
        children: [
          { path: '', redirect: { name: 'direct_members' } },
          {
            name: 'direct_members',
            path: '/group/:node_id/direct_members',
            component: () => import('components/groups/MembersTables.vue'),
          },
          {
            name: 'members_of',
            path: '/group/:node_id/member_of',
            component: () => import('components/groups/MemberOfTables.vue'),
          },
          {
            name: 'all_members',
            path: '/group/:node_id/all_members',
            component: () => import('components/groups/AllMemberTable.vue'),
          },
        ],
      },
      {
        path: '/tag/:node_id',
        component: () => import('pages/TagPage.vue'),
        props: true,
        children: [
          { path: '', redirect: { name: 'all_assets' } },
          {
            name: 'all_assets',
            path: '/tag/:node_id/all_assets',
            component: () => import('components/tags/AllAssets.vue'),
          },
          {
            name: 'direct_assets',
            path: '/tag/:node_id/direct_assets',
            component: () => import('components/tags/DirectlyTaggedAssets.vue'),
          },
          {
            name: 'user_access',
            path: '/tag/:node_id/users',
            component: () => import('components/tags/UserAccess.vue'),
          },
        ],
      },
      {
        path: '/asset/:node_id',
        component: () => import('pages/AssetPage.vue'),
        props: true,
        children: [
          { path: '', redirect: { name: 'users' } },
          {
            name: 'users',
            path: '/asset/:node_id/direct_access',
            component: () => import('components/assets/DirectAccess.vue'),
          },
          {
            path: '/asset/:node_id/any_access',
            component: () => import('components/assets/AnyAccess.vue'),
          },
          {
            path: '/asset/:node_id/hierarchy',
            component: () => import('components/assets/HierarchyTables.vue'),
          },
          {
            path: '/asset/:node_id/lineage',
            component: () => import('components/assets/LineageTables.vue'),
          },
        ],
      },
    ],
  },

  // Always leave this as last one,
  // but you can also remove it
  {
    path: '/:catchAll(.*)*',
    component: () => import('pages/ErrorNotFound.vue'),
  },
];

export default routes;
