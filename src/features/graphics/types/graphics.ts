export type GraphicsKind = "personPortrait" | "clubLogo" | "clubIcon";

export type GraphicsDiagnostics = {
  configLimit: number;
  entryLimit: number;
  depthLimit: number;
  mappingLimit: number;
  configTooLarge: number;
  configUnreadable: number;
  malformedConfig: number;
  invalidMapping: number;
  sourceUnreadable: number;
};

export type GraphicsStatus = {
  generation: number;
  selected: boolean;
  candidate: {
    available: boolean;
    source: "documents" | "onedrive" | "absent";
  };
  summary: {
    configs: number;
    mappings: number;
    truncated: boolean;
    diagnostics: GraphicsDiagnostics;
  };
};

export type GraphicsResult =
  | { status: "available"; bytes: number[]; mime: string }
  | { status: "missing" };
