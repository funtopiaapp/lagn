import { CharaView } from "./CharaView";
import { JaiminiView } from "./JaiminiView";
import { KpView } from "./KpView";
import { UpagrahaView } from "./UpagrahaView";
import { VarshaView } from "./VarshaView";
import { BalaView } from "./BalaView";
import { YoginiView } from "./YoginiView";
import { AshtakavargaView } from "./AshtakavargaView";
import type { BirthInput } from "../types";

/** The professional tab ids, so App and this module cannot drift apart. */
export type ProTab =
  | "jaimini" | "chara" | "upagraha" | "kp" | "varsha" | "bala" | "yogini" | "av";

/**
 * Every professional panel, in one module.
 *
 * This exists so the whole professional surface is a single lazily-loaded
 * chunk. Two things follow from that:
 *
 *  - a Lite reader never downloads it. Eight professional views is a lot of
 *    JavaScript to send someone who wants a reading in plain words.
 *  - when the owner's kill switch is off, the chunk is never requested at
 *    all, and the build drops it entirely. Gating only at render time left
 *    the code in the bundle, where anyone could read the feature names out
 *    of it - which is a weak sort of "off".
 */
export function ProPanels({ tab, birth }: { tab: ProTab; birth: BirthInput }) {
  switch (tab) {
    case "jaimini": return <JaiminiView birth={birth} />;
    case "chara": return <CharaView birth={birth} />;
    case "upagraha": return <UpagrahaView birth={birth} />;
    case "kp": return <KpView birth={birth} />;
    case "varsha": return <VarshaView birth={birth} />;
    case "bala": return <BalaView birth={birth} />;
    case "yogini": return <YoginiView birth={birth} />;
    case "av": return <AshtakavargaView birth={birth} />;
  }
}
