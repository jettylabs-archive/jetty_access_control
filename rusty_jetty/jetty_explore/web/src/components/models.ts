// Type definitions. Examples:
//
// export interface Todo {
//   id: number;
//   content: string;
// }

// export interface Meta {
//   totalCount: number;
// }

export type GroupPath = GroupSummary[];

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
