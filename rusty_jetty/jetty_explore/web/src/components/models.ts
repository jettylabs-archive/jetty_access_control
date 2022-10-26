// Type definitions. Examples:
//
// export interface Todo {
//   id: number;
//   content: string;
// }

// export interface Meta {
//   totalCount: number;
// }

export type NodePath = (GroupSummary | UserSummary)[];

export interface GroupName {
  Group: {
    name: string;
    origin: string;
  };
}

export interface GroupSummary {
  Group: {
    name: GroupName;
    connectors: string[];
  };
}

export interface UserName {
  User: string;
}

export interface UserSummary {
  User: {
    name: UserName;
    connectors: string[];
  };
}

export interface AssetName {
  Asset: {
    uri: string;
  };
}

export interface AssetSummary {
  Asset: {
    name: AssetName;
    asset_type: string;
    connectors: string[];
  };
}

export interface EffectivePermission {
  privilege: string;
  mode: 'Allow' | 'Deny';
  reasons: string[];
}
