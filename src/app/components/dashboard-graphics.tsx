import { PortraitInitials } from "@/features/dashboard/components/portrait";
import type { DashboardGraphics } from "@/features/dashboard/types/dashboard";
import { GraphicsImage } from "@/features/graphics/components/graphics-image";

/**
 * The faces and crests the dashboard's components show. Module-level, so their
 * cards keep one graphics identity across renders.
 */
export const dashboardGraphics: DashboardGraphics = {
  portrait: (uid, name) => (
    <GraphicsImage
      kind="personPortrait"
      uid={uid}
      slot={{
        alt: "",
        className: "size-full object-contain",
        fallback: <PortraitInitials name={name} />,
      }}
    />
  ),
  crest: (clubUid) => (
    <GraphicsImage
      kind="clubLogo"
      uid={clubUid ?? 0}
      slot={{ alt: "", className: "size-3 object-contain" }}
    />
  ),
};
