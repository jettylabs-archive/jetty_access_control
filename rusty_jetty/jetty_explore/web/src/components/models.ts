// Type definitions. Examples:
//
// export interface Todo {
//   id: number;
//   content: string;
// }

// export interface Meta {
//   totalCount: number;
// }

export type NodePath = AssetSummary[];

export interface GroupName {
  Group: {
    name: string;
    origin: string;
  };
}

export interface GroupSummary {
  Group: {
    name: GroupName;
    id: string;
    connectors: string[];
  };
}

export interface UserName {
  User: string;
}

export interface UserSummary {
  User: {
    name: UserName;
    id: string;
    connectors: string[];
  };
}

export interface AssetName {
  Asset: {
    asset_type?: string;
    connector: string;
    path: string[];
  };
}

export interface AssetSummary {
  Asset: {
    name: AssetName;
    id: string;
    asset_type: string;
    connectors: string[];
  };
}

export interface EffectivePermission {
  privilege: string;
  mode: 'Allow' | 'Deny';
  reasons: string[];
}

export interface TagName {
  Tag: string;
}

export interface TagSummary {
  Tag: {
    name: TagName;
    id: string;
    description: null | string;
    pass_through_hierarchy: boolean;
    pass_through_lineage: boolean;
    connectors: string[];
  };
}

export interface GroupWithPaths {
  node: GroupSummary;
  paths: NodePath[];
}

export interface AssetWithPaths {
  node: AssetSummary;
  paths: NodePath[];
}

// Defines the options for jettySearch.
export interface SearchOptions {
  caseSensitive?: boolean;
  numResults?: number;
}
