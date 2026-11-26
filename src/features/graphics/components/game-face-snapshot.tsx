import { createContext } from "react";

/**
 * The snapshot whose generated players `GraphicsImage` asks FM's own faces for.
 * The app shell provides the current snapshot; null means no game faces.
 */
export const GameFaceSnapshotContext = createContext<number | null>(null);
