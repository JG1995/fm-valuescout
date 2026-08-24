import { describe, expect, it, vi } from "vitest";
import { invokeCommand } from "@/lib/tauri-client";
import { clubDnaKeys } from "./club-dna-keys";
import { clubDnaQueryOptions, getClubDna } from "./club-dna-query-options";
import { removeClubDna } from "./remove-club-dna";
import { setClubDna } from "./set-club-dna";

vi.mock("@/lib/tauri-client", () => ({ invokeCommand: vi.fn() }));

const contextA = { saveId: 1, contextToken: "save-a" };
const sameSaveDifferentToken = { saveId: 1, contextToken: "save-b" };
const differentSaveSameToken = { saveId: 2, contextToken: "save-a" };

describe("Club DNA API", () => {
  it("keys each definition by its save ID and context token", () => {
    expect(clubDnaKeys.definition(contextA)).toEqual([
      "club-dna",
      "definition",
      { saveId: 1, contextToken: "save-a" },
    ]);
    expect(clubDnaKeys.definition(contextA)).not.toEqual(
      clubDnaKeys.definition(sameSaveDifferentToken),
    );
    expect(clubDnaKeys.definition(contextA)).not.toEqual(
      clubDnaKeys.definition(differentSaveSameToken),
    );
    expect(clubDnaQueryOptions(contextA).queryKey).toEqual([
      "club-dna",
      "definition",
      { saveId: 1, contextToken: "save-a" },
    ]);
  });

  it("passes the expected context to get, set, and remove", async () => {
    const invoke = vi.mocked(invokeCommand);
    invoke.mockResolvedValue(undefined);

    await getClubDna(contextA);
    await setClubDna(contextA, ["attr.Acceleration"]);
    await removeClubDna(contextA);

    expect(invoke).toHaveBeenNthCalledWith(1, "get_club_dna", contextA);
    expect(invoke).toHaveBeenNthCalledWith(2, "set_club_dna", {
      ...contextA,
      attributeIds: ["attr.Acceleration"],
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "remove_club_dna", contextA);
  });
});
