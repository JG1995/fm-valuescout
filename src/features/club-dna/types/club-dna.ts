export type ClubDnaContext = {
  saveId: number;
  contextToken: string;
};

export type ClubDnaDefinition = {
  attributeIds: string[];
};

export type ClubDnaUpsertResult = {
  definition: ClubDnaDefinition;
  created: boolean;
};

export type ClubDnaRemoveResult = {
  removed: boolean;
};
