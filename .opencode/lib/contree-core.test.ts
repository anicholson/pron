import { describe, it, expect } from "bun:test";
import {
  endsWithQuestion,
  validateMentalModel,
  buildDriftNudges,
  continuousStretchSeconds,
  shouldSelfCareNudge,
} from "./contree-core.ts";

describe("endsWithQuestion", () => {
  it("is true when the trimmed text ends with a question mark", () => {
    expect(endsWithQuestion("Should I proceed?")).toBe(true);
    expect(endsWithQuestion("Should I proceed?  \n")).toBe(true);
  });
  it("is false otherwise", () => {
    expect(endsWithQuestion("Done.")).toBe(false);
    expect(endsWithQuestion("")).toBe(false);
  });
});

describe("validateMentalModel", () => {
  const WELL_FORMED = [
    "Core Domain Identity",
    "World-to-Code Mapping",
    "Ubiquitous Language",
    "Bounded Contexts",
    "Invariants",
    "Decision Rationale",
    "Temporal View",
  ]
    .map((s) => `## ${s}\n\n- one item\n`)
    .join("\n");

  it("reports no issues for a well-formed file", () => {
    expect(validateMentalModel(WELL_FORMED)).toEqual([]);
  });
  it("flags a missing section", () => {
    expect(validateMentalModel(WELL_FORMED.replace("## Temporal View\n\n- one item\n", ""))).toContain(
      "Temporal View section is missing",
    );
  });
  it("flags a rogue heading", () => {
    expect(validateMentalModel(WELL_FORMED + "\n## Rogue\n\n- x\n")).toContain(
      "Rogue is a rogue heading, not one of the seven named sections",
    );
  });
  it("flags a section that exceeds its cap", () => {
    const overflow = "## Bounded Contexts\n\n" + Array.from({ length: 8 }, () => "- x").join("\n") + "\n";
    expect(validateMentalModel(overflow)).toContain("Bounded Contexts exceeds its cap of 7 (has 8 items)");
  });
  it("flags a missing file", () => {
    expect(validateMentalModel(null)).toEqual(["MENTAL_MODEL.md is missing"]);
  });
});

describe("buildDriftNudges", () => {
  it("nudges to consider mental-model and readme staleness when both exist", () => {
    const out = buildDriftNudges(true, true);
    expect(out).toContain("MENTAL MODEL: Did this task reveal");
    expect(out).toContain("Is the README out of date");
    expect(out).toContain("If nothing needs attention, reply 0.");
  });
  it("directs creation when the files are missing", () => {
    const out = buildDriftNudges(false, false);
    expect(out).toContain("MENTAL_MODEL.md is missing at the project root");
    expect(out).toContain("README.md is missing at the project root");
  });
});

describe("continuousStretchSeconds", () => {
  it("measures the unbroken run back from now, breaking on a gap over the threshold", () => {
    // now=1000; messages at 1000, 900, 700 (gaps 100, 200 ≤ 300) then 200 (gap 500 > 300 breaks)
    expect(continuousStretchSeconds([1000, 900, 700, 200], 1000, 300)).toBe(300);
  });
  it("is zero when there is only the current heartbeat", () => {
    expect(continuousStretchSeconds([1000], 1000, 300)).toBe(0);
  });
});

describe("shouldSelfCareNudge", () => {
  // heartbeats 300s apart (= the gap threshold, not exceeding it) → a continuous run
  const TWENTY_MIN = [0, 300, 600, 900, 1200];
  it("nudges after >= 20 min of continuous activity when not recently nudged", () => {
    expect(shouldSelfCareNudge(TWENTY_MIN, 1200, null, 1200, 300)).toBe(true);
  });
  it("does not nudge before 20 min of continuous activity", () => {
    expect(shouldSelfCareNudge([0, 300, 600], 600, null, 1200, 300)).toBe(false);
  });
  it("does not nudge again within the throttle window", () => {
    expect(shouldSelfCareNudge(TWENTY_MIN, 1200, 100, 1200, 300)).toBe(false); // last nudge 1100s ago < 1200
  });
});
